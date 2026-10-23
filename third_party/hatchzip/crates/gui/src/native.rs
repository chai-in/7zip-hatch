//! Ordinary archive operations use the bundled 7-Zip engine. Passwords go
//! through a private stdin pipe; filenames are arguments, never shell text.

use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use hatchzip_core::archive::Kind;
use hatchzip_core::session::SpacePlan;

use crate::entries::Entry;
use crate::platform;

pub fn binary() -> Result<PathBuf> {
    if cfg!(debug_assertions)
        && let Some(path) = std::env::var_os("SEVENZIP_HATCH_BINARY")
    {
        return Ok(path.into());
    }
    let exe = std::env::current_exe().context("locating 7-Zip Hatch")?;
    let path = exe.with_file_name(platform::ENGINE_NAME);
    if !path.is_file() {
        bail!("The bundled 7-Zip engine is missing. Reinstall 7-Zip Hatch.");
    }
    Ok(path)
}

fn command(binary: &Path, args: &[OsString]) -> Command {
    let mut c = Command::new(binary);
    c.arg(&args[0])
        .args(["-sccUTF-8", "-bse2"])
        .args(&args[1..]);
    platform::quiet_command(&mut c);
    c
}

fn read_stream(mut stream: impl Read, progress: Arc<dyn Fn(u8) + Send + Sync>) -> String {
    let mut text = Vec::new();
    let mut word = Vec::new();
    let mut buf = [0; 4096];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        for &b in &buf[..n] {
            if b == b'%' {
                if let Ok(p) = String::from_utf8_lossy(&word).parse::<u8>() {
                    progress(p.min(100));
                }
                word.clear();
            } else if b.is_ascii_digit() {
                if word.len() < 3 {
                    word.push(b);
                } else {
                    word.clear();
                }
            } else {
                word.clear();
            }
        }
        text.extend_from_slice(&buf[..n]);
        // Keep enough diagnostics, without retaining every filename in a
        // multi-million-entry archive operation.
        if text.len() > 128 * 1024 {
            text.drain(..text.len() - 64 * 1024);
        }
    }
    String::from_utf8_lossy(&text).into_owned()
}

fn execute(
    binary: &Path,
    args: Vec<OsString>,
    password: Option<&str>,
    input_archive: Option<&Path>,
    cancel: &AtomicBool,
    progress: Arc<dyn Fn(u8) + Send + Sync>,
) -> Result<String> {
    if password.is_some_and(|p| p.contains(['\n', '\r', '\0'])) {
        bail!("Passwords must not contain line breaks or a NUL character.");
    }
    let mut producer = if let Some(archive) = input_archive {
        let mut c = command(
            binary,
            &[
                "x".into(),
                "-so".into(),
                "-bso0".into(),
                "-bsp0".into(),
                "--".into(),
                archive.as_os_str().to_owned(),
            ],
        );
        Some(
            c.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .context("opening the compressed tar stream")?,
        )
    } else {
        None
    };
    let mut c = command(binary, &args);
    c.stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(p) = producer.as_mut() {
        c.stdin(p.stdout.take().expect("piped output"));
    } else if password.is_some() {
        c.stdin(Stdio::piped());
    } else {
        c.stdin(Stdio::null());
    }
    let mut child = match c.spawn() {
        Ok(c) => c,
        Err(e) => {
            if let Some(p) = producer.as_mut() {
                let _ = p.kill();
                let _ = p.wait();
            }
            return Err(e).context("starting the bundled 7-Zip engine");
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        let password = password.unwrap_or_default().to_owned();
        thread::spawn(move || {
            // Creation may request confirmation; encrypted readers request
            // one password. Never place the secret in process arguments.
            for _ in 0..3 {
                if writeln!(stdin, "{password}").is_err() {
                    break;
                }
            }
        });
    }
    let out = child.stdout.take().expect("piped stdout");
    let err = child.stderr.take().expect("piped stderr");
    let p = progress.clone();
    let out_thread = thread::spawn(move || read_stream(out, p));
    let err_thread = thread::spawn(move || read_stream(err, progress));
    let producer_errors = producer.as_mut().map(|p| {
        let err = p.stderr.take().expect("piped stderr");
        thread::spawn(move || read_stream(err, Arc::new(|_| {})))
    });
    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            if let Some(p) = producer.as_mut() {
                let _ = p.kill();
            }
        }
        if let Some(status) = child.try_wait()? {
            break status;
        }
        thread::sleep(Duration::from_millis(40));
    };
    let out = out_thread.join().unwrap_or_default();
    let err = err_thread.join().unwrap_or_default();
    let upstream_ok = if let Some(p) = producer.as_mut() {
        p.wait()?.success()
    } else {
        true
    };
    let upstream_err = producer_errors
        .map(|t| t.join().unwrap_or_default())
        .unwrap_or_default();
    if cancel.load(Ordering::Relaxed) {
        bail!("Operation cancelled. The source files were kept.");
    }
    if !status.success() || !upstream_ok {
        bail!(
            "7-Zip could not finish the operation.\n{}\n{}\n{}",
            err.trim(),
            upstream_err.trim(),
            out.trim()
        );
    }
    Ok(out)
}

pub struct Listing {
    pub(crate) entries: Vec<Entry>,
    pub size: u64,
}

pub fn list(archive: &Path, password: Option<&str>) -> Result<Listing> {
    let binary = binary()?;
    // Listings need the complete output rather than the diagnostic ring.
    let mut c = command(
        &binary,
        &["l".into(), "-slt".into(), "--".into(), archive.into()],
    );
    c.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = c.spawn().context("listing the archive with 7-Zip")?;
    if let Some(mut stdin) = child.stdin.take()
        && let Some(p) = password
    {
        if p.contains(['\n', '\r', '\0']) {
            let _ = child.kill();
            let _ = child.wait();
            bail!("Invalid password characters.");
        }
        let p = p.to_owned();
        thread::spawn(move || {
            let _ = writeln!(stdin, "{p}");
        });
    }
    let output = child.wait_with_output()?;
    let text = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    if !output.status.success() {
        bail!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr).trim(),
            text.trim()
        );
    }
    let body = text
        .split_once("\n----------\n")
        .map(|(_, b)| b)
        .ok_or_else(|| anyhow!("7-Zip returned an unreadable archive listing"))?;
    let mut entries = Vec::new();
    let mut size = 0_u64;
    for block in body.split("\n\n") {
        let fields: std::collections::HashMap<_, _> =
            block.lines().filter_map(|l| l.split_once(" = ")).collect();
        let Some(name) = fields.get("Path") else {
            continue;
        };
        let dir = fields.get("Folder") == Some(&"+")
            || fields.get("Attributes").is_some_and(|s| s.starts_with('D'));
        let bytes = fields
            .get("Size")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        size = size
            .checked_add(bytes)
            .context("archive sizes exceed the supported range")?;
        entries.push(Entry {
            path: (*name).into(),
            size: bytes,
            kind: if dir { Kind::Dir } else { Kind::File },
        });
    }
    Ok(Listing { entries, size })
}

pub fn plan(size: u64, output: &Path, keep_free: u64) -> Result<SpacePlan> {
    let mut parent = output;
    while !parent.exists() {
        parent = parent.parent().context("invalid destination folder")?;
    }
    let free = fs4::available_space(parent)?;
    let used = size
        .checked_add(keep_free)
        .context("reserved space is too large")?;
    Ok(SpacePlan {
        free,
        needed: size,
        keep_free,
        fits: free >= used,
        free_after: free.saturating_sub(size),
        normal_needs: size,
        reclaims: true,
        solid_block_bound: false,
        archive_kept_whole: false,
    })
}

#[derive(Clone)]
pub struct PackOptions {
    pub format: String,
    pub level: u8,
    pub method: String,
    pub dictionary: Option<String>,
    pub solid: String,
    pub threads: String,
    pub volume: String,
    pub password: String,
    pub encrypt_headers: bool,
    pub advanced: String,
}

impl PackOptions {
    pub fn validate(&self) -> Result<()> {
        if self.password.contains(['\n', '\r', '\0']) {
            bail!("Passwords must not contain line breaks or a NUL character.");
        }
        // The native ZIP writer rejects non-ASCII passwords and limits AES
        // passwords to 99 bytes. Explain that restriction before starting.
        if self.format == "zip"
            && (self.password.len() > 99
                || self.password.bytes().any(|b| !(0x20..=0x7e).contains(&b)))
        {
            bail!(
                "ZIP passwords must use at most 99 printable ASCII characters. Choose 7z for a Unicode password."
            );
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum Operation {
    Extract {
        archive: PathBuf,
        output: PathBuf,
        password: Option<String>,
    },
    Test {
        archive: PathBuf,
        password: Option<String>,
    },
    Create {
        destination: PathBuf,
        inputs: Vec<PathBuf>,
        options: PackOptions,
    },
}

pub struct Finished {
    pub message: String,
    pub folder: PathBuf,
    pub archive: Option<PathBuf>,
}

impl Operation {
    pub fn run(
        &self,
        cancel: &AtomicBool,
        progress: Arc<dyn Fn(u8) + Send + Sync>,
    ) -> Result<Finished> {
        let binary = binary()?;
        match self {
            Self::Test { archive, password } => {
                execute(
                    &binary,
                    vec!["t".into(), "-bsp1".into(), "--".into(), archive.into()],
                    password.as_deref(),
                    None,
                    cancel,
                    progress,
                )?;
                Ok(Finished {
                    message: "Archive integrity test passed.".into(),
                    folder: archive.parent().unwrap_or(Path::new(".")).into(),
                    archive: None,
                })
            }
            Self::Extract {
                archive,
                output,
                password,
            } => {
                if output.exists() && (!output.is_dir() || fs::read_dir(output)?.next().is_some()) {
                    bail!(
                        "The destination is not empty. Choose a new or empty folder to keep existing files safe."
                    );
                }
                let parent = output.parent().context("choose a destination folder")?;
                fs::create_dir_all(parent)?;
                let stage = tempfile::Builder::new()
                    .prefix(".7zip-hatch-extract-")
                    .tempdir_in(parent)?;
                let name = archive.to_string_lossy().to_ascii_lowercase();
                let stream_tar = [
                    ".tar.gz", ".tgz", ".tar.xz", ".txz", ".tar.bz2", ".tbz2", ".tar.zst", ".tzst",
                ]
                .iter()
                .any(|ext| name.ends_with(ext));
                let mut args: Vec<OsString> = vec![
                    "x".into(),
                    "-y".into(),
                    "-bsp1".into(),
                    format!("-o{}", stage.path().display()).into(),
                ];
                if stream_tar {
                    args.extend(["-si".into(), "-ttar".into()]);
                } else {
                    args.extend(["--".into(), archive.into()]);
                }
                execute(
                    &binary,
                    args,
                    password.as_deref(),
                    stream_tar.then_some(archive.as_path()),
                    cancel,
                    progress,
                )?;
                if output.exists() {
                    fs::remove_dir(output).context("the destination changed during extraction; extracted files were not published")?;
                }
                fs::rename(stage.path(), output)
                    .context("moving extracted files into the destination")?;
                Ok(Finished {
                    message: "Extraction complete. The original archive was kept.".into(),
                    folder: output.clone(),
                    archive: None,
                })
            }
            Self::Create {
                destination,
                inputs,
                options,
            } => {
                options.validate()?;
                if inputs.is_empty() {
                    bail!("Choose files or a folder to compress.");
                }
                if destination.exists() {
                    bail!("That archive already exists. Choose another name.");
                }
                if !["7z", "zip"].contains(&options.format.as_str()) {
                    bail!("Choose 7z or ZIP.");
                }
                let parent = destination.parent().context("choose an archive location")?;
                fs::create_dir_all(parent)?;
                let stage = tempfile::Builder::new()
                    .prefix(".7zip-hatch-create-")
                    .tempdir_in(parent)?;
                let name = destination
                    .file_name()
                    .context("choose an archive filename")?;
                let target = stage.path().join(name);
                let mut args: Vec<OsString> = vec![
                    "a".into(),
                    format!("-t{}", options.format).into(),
                    format!("-mx{}", options.level).into(),
                    format!("-mmt={}", options.threads).into(),
                    "-bsp1".into(),
                    format!(
                        "-xr!{}",
                        stage.path().file_name().unwrap().to_string_lossy()
                    )
                    .into(),
                ];
                args.push(
                    format!(
                        "-m{}={}",
                        if options.format == "7z" { "0" } else { "m" },
                        options.method
                    )
                    .into(),
                );
                if let Some(dict) = &options.dictionary {
                    args.push(format!("-md={dict}").into());
                }
                if options.format == "7z" {
                    args.push(format!("-ms={}", options.solid).into());
                }
                if !options.password.is_empty() {
                    args.push("-p".into());
                    if options.format == "7z" {
                        args.push(
                            format!(
                                "-mhe={}",
                                if options.encrypt_headers { "on" } else { "off" }
                            )
                            .into(),
                        );
                    } else {
                        args.push("-mem=AES256".into());
                    }
                }
                if !options.volume.trim().is_empty() {
                    let v = options.volume.trim();
                    let digits = v.trim_end_matches(|c: char| c.is_ascii_alphabetic());
                    let unit = &v[digits.len()..];
                    if digits.parse::<u64>().ok().filter(|n| *n > 0).is_none()
                        || !["", "b", "k", "m", "g"].contains(&unit.to_ascii_lowercase().as_str())
                    {
                        bail!("Volume size must look like 256m or 2g.");
                    }
                    args.push(format!("-v{v}").into());
                }
                for property in options.advanced.split_whitespace() {
                    if !property.starts_with("-m") {
                        bail!(
                            "Advanced compression properties must start with -m, for example -m0=BCJ -m1=LZMA2."
                        );
                    }
                    args.push(property.into());
                }
                args.extend([target.into_os_string(), "--".into()]);
                args.extend(inputs.iter().map(|p| p.as_os_str().to_owned()));
                execute(
                    &binary,
                    args,
                    (!options.password.is_empty()).then_some(options.password.as_str()),
                    None,
                    cancel,
                    progress,
                )?;
                let mut files: Vec<_> = fs::read_dir(stage.path())?
                    .map(|e| e.map(|e| e.path()))
                    .collect::<std::io::Result<_>>()?;
                files.sort();
                if files.is_empty() {
                    bail!("7-Zip did not create an archive.");
                }
                let mut published = Vec::new();
                for file in &files {
                    let dest = parent.join(file.file_name().context("invalid output filename")?);
                    let temporary = tempfile::TempPath::try_from_path(file.clone())?;
                    if let Err(e) = temporary.persist_noclobber(&dest) {
                        for p in &published {
                            let _ = fs::remove_file(p);
                        }
                        return Err(e)
                            .context("publishing the archive without overwriting existing files");
                    }
                    published.push(dest);
                }
                Ok(Finished {
                    message: "Archive created. The source files were kept.".into(),
                    folder: parent.into(),
                    archive: published.into_iter().next(),
                })
            }
        }
    }
}
