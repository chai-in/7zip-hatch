//! Real archive operations through the engine shipped beside the desktop app.
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use hatchzip_gui::native::{self, Operation, PackOptions};

fn options(format: &str) -> PackOptions {
    PackOptions {
        format: format.into(),
        level: 1,
        method: if format == "7z" { "LZMA2" } else { "Deflate" }.into(),
        dictionary: None,
        solid: "64m".into(),
        threads: "2".into(),
        volume: String::new(),
        password: String::new(),
        encrypt_headers: true,
        advanced: String::new(),
    }
}

fn run(operation: Operation) -> native::Finished {
    operation
        .run(&AtomicBool::new(false), Arc::new(|_| {}))
        .unwrap()
}

fn fixture(root: &Path) -> std::path::PathBuf {
    let input = root.join("input with spaces");
    fs::create_dir(&input).unwrap();
    fs::write(
        input.join("naïve 日本語.txt"),
        b"actual desktop archive content",
    )
    .unwrap();
    fs::write(
        input.join("data.bin"),
        (0_u8..=255).cycle().take(200_000).collect::<Vec<_>>(),
    )
    .unwrap();
    fs::create_dir(input.join("empty folder")).unwrap();
    input
}

#[test]
fn creates_extracts_and_tests_encrypted_archives_without_consuming_inputs() {
    assert!(
        native::binary().unwrap().is_file(),
        "build the bundled engine before the GUI tests"
    );
    for format in ["7z", "zip"] {
        let tmp = tempfile::tempdir().unwrap();
        let input = fixture(tmp.path());
        let archive = tmp.path().join(format!("desktop.{format}"));
        let mut settings = options(format);
        settings.password = if format == "7z" {
            "disposable 日本語 password"
        } else {
            "disposable ZIP password"
        }
        .into();
        run(Operation::Create {
            destination: archive.clone(),
            inputs: vec![input.clone()],
            options: settings.clone(),
        });
        assert!(input.join("data.bin").exists());
        let before = fs::read(&archive).unwrap();
        let listing = native::list(&archive, Some(&settings.password)).unwrap();
        assert!(listing.size >= 200_000);
        run(Operation::Test {
            archive: archive.clone(),
            password: Some(settings.password.clone()),
        });
        let output = tmp.path().join("extracted");
        run(Operation::Extract {
            archive: archive.clone(),
            output: output.clone(),
            password: Some(settings.password),
        });
        assert_eq!(fs::read(&archive).unwrap(), before);
        assert_eq!(
            fs::read(output.join("input with spaces/data.bin")).unwrap(),
            fs::read(input.join("data.bin")).unwrap()
        );
        assert!(output.join("input with spaces/empty folder").is_dir());
    }
}

#[test]
fn publishes_split_volumes_and_refuses_to_overwrite_files() {
    let tmp = tempfile::tempdir().unwrap();
    let input = fixture(tmp.path());
    // Incompressible data makes several small volumes.
    let mut state = 17_u64;
    let noise: Vec<u8> = (0..200_000)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect();
    fs::write(input.join("noise"), noise).unwrap();
    let destination = tmp.path().join("split.7z");
    let mut settings = options("7z");
    settings.volume = "64k".into();
    let made = run(Operation::Create {
        destination,
        inputs: vec![input.clone()],
        options: settings,
    });
    let first = made.archive.unwrap();
    assert!(first.to_string_lossy().ends_with(".001"));
    assert!(first.with_extension("002").exists());
    let out = tmp.path().join("output");
    run(Operation::Extract {
        archive: first.clone(),
        output: out.clone(),
        password: None,
    });
    let refused = Operation::Extract {
        archive: first,
        output: out.clone(),
        password: None,
    }
    .run(&AtomicBool::new(false), Arc::new(|_| {}));
    assert!(refused.is_err());
    assert_eq!(
        fs::read(out.join("input with spaces/noise")).unwrap(),
        fs::read(input.join("noise")).unwrap()
    );
}

#[test]
fn cancellation_and_invalid_properties_keep_originals_and_remove_staging_files() {
    let tmp = tempfile::tempdir().unwrap();
    let input = fixture(tmp.path());
    let archive = tmp.path().join("cancel.zip");
    run(Operation::Create {
        destination: archive.clone(),
        inputs: vec![input.clone()],
        options: options("zip"),
    });
    let before = fs::read(&archive).unwrap();
    let output = tmp.path().join("output");
    assert!(
        Operation::Extract {
            archive: archive.clone(),
            output: output.clone(),
            password: None
        }
        .run(&AtomicBool::new(true), Arc::new(|_| {}))
        .is_err()
    );
    assert_eq!(fs::read(&archive).unwrap(), before);
    assert!(!output.exists());
    let mut bad = options("7z");
    bad.advanced = "-sdel".into();
    assert!(
        Operation::Create {
            destination: tmp.path().join("bad.7z"),
            inputs: vec![input.clone()],
            options: bad
        }
        .run(&AtomicBool::new(false), Arc::new(|_| {}))
        .is_err()
    );
    assert!(input.join("data.bin").exists());
    assert!(!fs::read_dir(tmp.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".7zip-hatch-")
    }));
}

#[test]
fn compressed_tar_extracts_directly_to_files_and_archives_can_be_created_inside_input() {
    let tmp = tempfile::tempdir().unwrap();
    let input = fixture(tmp.path());
    let archive = tmp.path().join("files.tar.gz");
    let engine = native::binary().unwrap();
    let tar = tmp.path().join("files.tar");
    assert!(
        std::process::Command::new(&engine)
            .args(["a", "-ttar"])
            .arg(&tar)
            .arg(&input)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        std::process::Command::new(&engine)
            .args(["a", "-tgzip"])
            .arg(&archive)
            .arg(&tar)
            .output()
            .unwrap()
            .status
            .success()
    );
    let output = tmp.path().join("out");
    run(Operation::Extract {
        archive: archive.clone(),
        output: output.clone(),
        password: None,
    });
    assert_eq!(
        fs::read(output.join("input with spaces/data.bin")).unwrap(),
        fs::read(input.join("data.bin")).unwrap()
    );
    assert!(archive.exists());
    let nested = input.join("inside.7z");
    run(Operation::Create {
        destination: nested.clone(),
        inputs: vec![input],
        options: options("7z"),
    });
    let listing = native::list(&nested, None).unwrap();
    assert!(
        listing.size < 300_000,
        "the staging archive must not archive itself"
    );
}
