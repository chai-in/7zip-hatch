mod cli;

use std::ffi::OsString;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anstream::{eprintln, print, println};
use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser};
use indicatif::{ProgressBar, ProgressStyle};

use cli::{
    Cli, Command, EXIT_DAMAGED, EXIT_FAILURE, EXIT_NO_SPACE, EXIT_PASSWORD, EXIT_PAUSED, Secret,
    Space, Target,
};
use hatchzip_core::archive::{Damaged, Kind, PasswordNeeded, WrongPassword};
use hatchzip_core::engine::{self, Job, Outcome, Progress, Run};
use hatchzip_core::human;
use hatchzip_core::journal::{Journal, Paths};
use hatchzip_core::session::{self, State, display_name, locate};

const ERROR: Style = AnsiColor::Red.on_default().bold();
const GOOD: Style = AnsiColor::Green.on_default().bold();
const BOLD: Style = Style::new().bold();
const DIM: Style = Style::new().dimmed();

static STOP: AtomicBool = AtomicBool::new(false);
static QUIET: AtomicBool = AtomicBool::new(false);

/// Normal output, silenced by `--quiet`. Errors and prompts always show.
macro_rules! say {
    ($($t:tt)*) => {
        if !QUIET.load(Ordering::Relaxed) {
            println!($($t)*);
        }
    };
}

/// Runs the CLI with explicit arguments so 7-Zip can embed the same engine.
pub fn main_from(args: impl IntoIterator<Item = OsString>) -> u8 {
    let cli = Cli::parse_from(args);
    QUIET.store(cli.quiet, Ordering::Relaxed);
    match run(cli.command) {
        Ok(code) => code,
        Err(e) if e.is::<PasswordNeeded>() || e.is::<WrongPassword>() => {
            eprintln!("{ERROR}error:{ERROR:#} {e:#}");
            if !io::stdin().is_terminal() {
                eprintln!(
                    "Pass --password or set HATCHZIP_PASSWORD when not running in a terminal."
                );
            }
            EXIT_PASSWORD
        }
        Err(e) if e.is::<Damaged>() => {
            eprintln!("{ERROR}error:{ERROR:#} {e:#}");
            EXIT_DAMAGED
        }
        Err(e) => {
            eprintln!("{ERROR}error:{ERROR:#} {e:#}");
            EXIT_FAILURE
        }
    }
}

/// Opens the archive, asking for a password when it is encrypted and none
/// was given, and again (up to three times) when one was wrong. Nothing is
/// touched until the password is proven.
fn open(
    archive: &Path,
    output: Option<std::path::PathBuf>,
    writable: bool,
    secret: Secret,
) -> Result<(State, Paths)> {
    let (primary, paths) = locate(archive)?;
    // A compressed tar is read in full before anything else, which can take
    // a while for a big one, so say what is happening.
    let name = primary.file_name().unwrap_or_default().to_string_lossy();
    let lower = name.to_ascii_lowercase();
    if (lower.ends_with(".tar.gz") || lower.ends_with(".tgz"))
        && io::stderr().is_terminal()
        && !QUIET.load(Ordering::Relaxed)
    {
        eprintln!("Verifying {name}…");
    }
    let mut tries = 0;
    let (state, _) = session::open_encrypted(
        &primary,
        &paths,
        output,
        writable,
        secret.password,
        |wrong| {
            if wrong {
                eprintln!("{ERROR}Incorrect password.{ERROR:#}");
            }
            tries += 1;
            if tries > 3 || !io::stdin().is_terminal() {
                return None;
            }
            rpassword::prompt_password("Password: ").ok()
        },
    )?;
    Ok((state, paths))
}

fn run(command: Command) -> Result<u8> {
    match command {
        Command::Extract {
            target,
            verify_first,
            space,
            yes,
        } => extract(target, verify_first, space, yes),
        Command::Check { target, space } => check(target, space),
        Command::List { archive, secret } => list(&archive, secret),
        Command::Test { archive, secret } => test_command(&archive, secret),
        Command::Restore { archive } => restore(&archive),
        Command::Completions { shell } => {
            // Named after however we were called, so `hz completions bash`
            // completes the `hz` alias.
            let name = std::env::args_os()
                .next()
                .and_then(|a| {
                    Path::new(&a)
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                })
                .filter(|n| n == "hz")
                .unwrap_or_else(|| "hatchzip".into());
            clap_complete::generate(shell, &mut Cli::command(), name, &mut io::stdout());
            Ok(0)
        }
        Command::Man => {
            clap_mangen::Man::new(Cli::command()).render(&mut io::stdout())?;
            Ok(0)
        }
    }
}

/// Prints the space plan and returns whether it fits. The verdict prints
/// even under `--quiet`, since it is the whole answer.
fn report(job: &Job, out: &Path, journal: Option<&Journal>, keep_free: u64) -> Result<bool> {
    let plan = session::plan(job, out, journal.is_some(), keep_free)?;
    let left = job.remaining_size();

    say!(
        "{BOLD}{}{BOLD:#}  {} {}, {} uncompressed in {}",
        display_name(job),
        human(job.storage.original_len()),
        job.format_name,
        human(journal.map_or(left, |j| j.total_size)),
        entries(job.item_count())
    );
    if let Some(j) = journal {
        say!(
            "  resuming: {} of {} remaining",
            human(left),
            human(j.total_size)
        );
    }
    say!("  available                    {}", human(plan.free));
    say!(
        "  standard extraction needs    {}",
        human(plan.normal_needs)
    );
    say!(
        "  required                     {BOLD}{}{BOLD:#} {DIM}(+{} reserved){DIM:#}",
        human(plan.needed),
        human(plan.keep_free)
    );
    if !plan.reclaims {
        say!(
            "  {DIM}note: the destination is on another drive, so removing the archive does not free space there{DIM:#}"
        );
    }
    if plan.archive_kept_whole {
        say!(
            "  {DIM}note: this drive cannot release space inside a file, so the compressed archive remains whole until extraction completes and room is required for the archive and every extracted file{DIM:#}"
        );
    }
    if plan.fits {
        say!("  available after              {}", human(plan.free_after));
    } else {
        let short = human(plan.needed + plan.keep_free - plan.free);
        println!(
            "{ERROR}Insufficient disk space:{ERROR:#} {short} more is required. Free up space or lower --keep-free."
        );
        if plan.solid_block_bound {
            println!(
                "{DIM}Most of the required space is one solid block, which can only be released after \
                 it is fully extracted. Archives created with smaller solid blocks require much \
                 less space.{DIM:#}"
            );
        }
    }
    Ok(plan.fits)
}

fn check(target: Target, space: Space) -> Result<u8> {
    let (state, _) = open(&target.archive, target.output, false, target.secret)?;
    let fits = match state {
        State::Fresh(job, out) => report(&job, &out, None, space.keep_free)?,
        State::Resume(job, journal) => {
            report(&job, &journal.output, Some(&journal), space.keep_free)?
        }
        State::Orphan(_) => {
            say!("All entries are already extracted. Run extract to complete the cleanup.");
            true
        }
    };
    Ok(if fits { 0 } else { EXIT_NO_SPACE })
}

fn list(archive: &Path, secret: Secret) -> Result<u8> {
    let job = match open(archive, None, false, secret)?.0 {
        State::Fresh(job, _) | State::Resume(job, _) => job,
        State::Orphan(_) => bail!("no entries remain in this archive"),
    };
    println!("{BOLD}{:>10}  Name{BOLD:#}", "Size");
    for item in job.units.iter().rev().flat_map(|u| &u.items) {
        let path = item.path.display();
        match &item.kind {
            Kind::Dir => println!("{:>10}  {path}/", ""),
            Kind::File => println!("{:>10}  {path}", human(item.size)),
            Kind::Symlink => println!("{:>10}  {path} {DIM}(symlink){DIM:#}", ""),
            Kind::HardLink(t) => {
                println!("{:>10}  {path} {DIM}(link to {}){DIM:#}", "", t.display())
            }
        }
    }
    say!(
        "{DIM}{}, {} uncompressed, {} {} archive{DIM:#}",
        entries(job.item_count()),
        human(job.remaining_size()),
        human(job.storage.original_len()),
        job.format_name
    );
    Ok(0)
}

fn test_command(archive: &Path, secret: Secret) -> Result<u8> {
    let job = match open(archive, None, false, secret)?.0 {
        State::Fresh(job, _) => job,
        _ => bail!(
            "this archive is partially extracted and cannot be tested. Run extract to resume it, or restore to undo it"
        ),
    };
    if let Err(e) = verify(&job) {
        eprintln!("{ERROR}error:{ERROR:#} {e:#}");
        return Ok(EXIT_DAMAGED);
    }
    say!(
        "{GOOD}Test passed:{GOOD:#} {} verified.",
        entries(job.item_count())
    );
    Ok(0)
}

fn entries(n: usize) -> String {
    if n == 1 {
        "1 entry".into()
    } else {
        format!("{n} entries")
    }
}

fn restore(archive: &Path) -> Result<u8> {
    let r = hatchzip_core::restore::restore(archive)?;
    let name = r.archive.file_name().unwrap_or_default().to_string_lossy();
    say!(
        "{GOOD}Archive restored.{GOOD:#} {name} is a standard {} archive again and contains \
         the {} not yet extracted.",
        r.format,
        entries(r.entries)
    );
    say!("Extracted files remain in {}.", r.output.display());
    Ok(0)
}

fn extract(target: Target, verify_first: bool, space: Space, yes: bool) -> Result<u8> {
    let keep_free = space.keep_free;
    let (state, paths) = open(&target.archive, target.output, true, target.secret)?;
    let (job, journal) = match state {
        State::Orphan(journal) => {
            engine::finish_orphaned(&journal, &paths)?;
            say!(
                "{GOOD}Extraction complete.{GOOD:#} Files are in {}.",
                journal.output.display()
            );
            return Ok(0);
        }
        State::Resume(job, journal) => {
            if !report(&job, &journal.output, Some(&journal), keep_free)? {
                return Ok(EXIT_NO_SPACE);
            }
            session::clear_uncommitted(&job, &journal.output)?;
            (job, journal)
        }
        State::Fresh(job, out) => {
            if !report(&job, &out, None, keep_free)? {
                return Ok(EXIT_NO_SPACE);
            }
            if verify_first && let Err(e) = verify(&job) {
                eprintln!("{ERROR}error:{ERROR:#} {e:#}");
                return Ok(EXIT_DAMAGED);
            }
            if !yes {
                confirm(&display_name(&job), &out)?;
            }
            let journal = session::begin(&job, &paths, out)?;
            (job, journal)
        }
    };

    ctrlc::set_handler(|| {
        if STOP.swap(true, Ordering::Relaxed) {
            std::process::exit(EXIT_PAUSED.into());
        }
        eprintln!(
            "\nPausing after the current batch. Partly written files are redone on resume. \
             Press Ctrl+C again to exit immediately."
        );
    })
    .context("installing Ctrl+C handler")?;

    let bar = progress_bar(journal.total_size, "Extracting");
    bar.set_position(journal.total_size - job.remaining_size());
    bar.set_message(format!("archive: {}", human(journal.committed_len)));

    let output = journal.output.clone();
    let count = job.item_count();
    let (b1, b2) = (bar.clone(), bar.clone());
    let run = Run {
        job,
        journal,
        paths,
        stop: &STOP,
        progress: Progress {
            on_bytes: Box::new(move |n| b1.inc(n)),
            on_commit: Box::new(move |len| b2.set_message(format!("archive: {}", human(len)))),
        },
        keep_free,
    };
    let outcome = run.execute();
    bar.finish_and_clear();

    match outcome {
        Ok(Outcome::Finished) => {
            say!(
                "{GOOD}Extraction complete:{GOOD:#} {} in {}. The archive was removed.",
                entries(count),
                output.display()
            );
            Ok(0)
        }
        Ok(Outcome::Paused) => {
            say!("Extraction paused. Run the same command to resume.");
            Ok(EXIT_PAUSED)
        }
        Err(e) => {
            eprintln!("{ERROR}error:{ERROR:#} {e:#}");
            eprintln!("Progress is saved. Run the same command to resume.");
            Ok(EXIT_FAILURE)
        }
    }
}

fn verify(job: &Job) -> Result<()> {
    let bar = progress_bar(job.remaining_size(), "Verifying");
    let b = bar.clone();
    let result = job.verify(&move |n| b.inc(n));
    bar.finish_and_clear();
    result.context("the archive is damaged; no files were extracted")
}

fn confirm(archive: &str, output: &Path) -> Result<()> {
    if !io::stdin().is_terminal() {
        bail!("HatchZip removes the archive during extraction; pass --yes to confirm");
    }
    print!(
        "\nExtract to {} and remove {BOLD}{archive}{BOLD:#} during extraction? [y/N] ",
        output.display()
    );
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    if !matches!(answer.trim(), "y" | "Y" | "yes") {
        bail!("extraction cancelled. The archive was not modified");
    }
    Ok(())
}

fn progress_bar(total: u64, prefix: &'static str) -> ProgressBar {
    if QUIET.load(Ordering::Relaxed) {
        return ProgressBar::hidden();
    }
    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::with_template(
            "{prefix} [{bar:32}] {binary_bytes}/{binary_total_bytes}  {binary_bytes_per_sec}  eta {eta}  {msg}",
        )
        .expect("valid template")
        .progress_chars("=> "),
    );
    bar.set_prefix(prefix);
    bar.enable_steady_tick(Duration::from_millis(200));
    bar
}

/// Called once, before 7-Zip installs its signal handlers. A negative result
/// leaves the command entirely to 7-Zip. Rust reads native OS arguments, so
/// filenames do not make a lossy round trip through a C string.
#[unsafe(no_mangle)]
pub extern "C" fn sevenzip_hatch_dispatch() -> i32 {
    let mut args = std::env::args_os();
    args.next();
    if args.next().as_deref() != Some(std::ffi::OsStr::new("hatch")) {
        return -1;
    }
    // No panic may unwind through the C++ entry point.
    match std::panic::catch_unwind(|| {
        let args = std::iter::once(OsString::from("7zz hatch")).chain(args);
        i32::from(main_from(args))
    }) {
        Ok(code) => code,
        Err(_) => {
            eprintln!(
                "HatchZip stopped unexpectedly. Keep the archive and its .hatchzip side files to resume."
            );
            i32::from(EXIT_FAILURE)
        }
    }
}
