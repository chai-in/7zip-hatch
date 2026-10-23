//! Native 7-Zip jobs and the archive creation dialog.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use slint::{CloseRequestResponse, ComponentHandle};

use crate::native::{Operation, PackOptions};
use crate::{CreateWindow, NativeTaskWindow};

pub fn task(
    heading: &str,
    detail: &str,
    operation: Operation,
    done: impl FnOnce(Option<PathBuf>) + Send + 'static,
) -> Result<NativeTaskWindow> {
    let ui = NativeTaskWindow::new()?;
    ui.set_heading(heading.into());
    ui.set_detail(detail.into());
    let cancel = Arc::new(AtomicBool::new(false));
    let c = cancel.clone();
    let weak = ui.as_weak();
    ui.on_cancel(move || {
        c.store(true, Ordering::Relaxed);
        if let Some(w) = weak.upgrade() {
            w.set_message("Stopping safely…".into());
        }
    });
    let weak = ui.as_weak();
    ui.on_dismiss(move || {
        if let Some(w) = weak.upgrade() {
            let _ = w.hide();
        }
    });
    let c = cancel.clone();
    let weak = ui.as_weak();
    ui.window().on_close_requested(move || {
        if weak.upgrade().is_some_and(|w| w.get_complete()) {
            CloseRequestResponse::HideWindow
        } else {
            c.store(true, Ordering::Relaxed);
            if let Some(w) = weak.upgrade() {
                w.set_message("Stopping safely…".into());
            }
            CloseRequestResponse::KeepWindowShown
        }
    });
    let weak = ui.as_weak();
    std::thread::spawn(move || {
        let p = weak.clone();
        let progress = Arc::new(move |percent: u8| {
            let p = p.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = p.upgrade() {
                    w.set_fraction(f32::from(percent) / 100.0);
                    w.set_message(format!("{percent}% complete").into());
                }
            });
        });
        let result = operation.run(&cancel, progress);
        let _ = slint::invoke_from_event_loop(move || {
            let mut archive = None;
            if let Some(w) = weak.upgrade() {
                w.set_complete(true);
                match result {
                    Ok(result) => {
                        w.set_fraction(1.0);
                        w.set_message(result.message.into());
                        w.on_open_folder(move || {
                            let _ = open::that_detached(&result.folder);
                        });
                        archive = result.archive;
                    }
                    Err(e) => {
                        w.set_failed(true);
                        w.set_message(format!("{e:#}").into());
                    }
                }
            }
            done(archive);
        });
    });
    ui.show()?;
    Ok(ui)
}

pub fn create(inputs: Vec<PathBuf>, submit: impl Fn(Operation) + 'static) -> Result<CreateWindow> {
    let ui = CreateWindow::new()?;
    ui.set_inputs(
        inputs
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .into(),
    );
    let parent = inputs
        .first()
        .and_then(|p| p.parent())
        .unwrap_or(std::path::Path::new("."));
    ui.set_destination(parent.join("Archive.7z").display().to_string().into());
    let weak = ui.as_weak();
    ui.on_cancel(move || {
        if let Some(w) = weak.upgrade() {
            let _ = w.hide();
        }
    });
    let weak = ui.as_weak();
    ui.on_format_changed(move || {
        if let Some(w) = weak.upgrade() {
            let path = PathBuf::from(w.get_destination().as_str());
            w.set_destination(
                path.with_extension(if w.get_format_index() == 0 {
                    "7z"
                } else {
                    "zip"
                })
                .display()
                .to_string()
                .into(),
            );
        }
    });
    let weak = ui.as_weak();
    ui.on_browse(move || {
        let Some(w) = weak.upgrade() else { return };
        let path = PathBuf::from(w.get_destination().as_str());
        let dialog = rfd::AsyncFileDialog::new()
            .set_title("Save archive")
            .set_file_name(path.file_name().unwrap_or_default().to_string_lossy())
            .set_directory(path.parent().unwrap_or(std::path::Path::new(".")))
            .set_parent(&w.window().window_handle());
        let weak = w.as_weak();
        let _ = slint::spawn_local(async move {
            if let Some(file) = dialog.save_file().await
                && let Some(w) = weak.upgrade()
            {
                w.set_destination(file.path().display().to_string().into());
            }
        });
    });
    let weak = ui.as_weak();
    ui.on_create(move || {
        let Some(w) = weak.upgrade() else { return };
        let destination = PathBuf::from(w.get_destination().as_str());
        if !destination.is_absolute() || destination.file_name().is_none() {
            w.set_error("Choose a full archive path using Save as.".into());
            return;
        }
        let zip = w.get_format_index() == 1;
        let methods = if zip {
            ["Deflate", "BZip2", "LZMA", "PPMd", "Copy"]
        } else {
            ["LZMA2", "LZMA", "BZip2", "PPMd", "Copy"]
        };
        let method = methods[w.get_method_index().clamp(0, 4) as usize];
        let dict =
            ["", "8m", "16m", "32m", "64m", "128m"][w.get_dictionary_index().clamp(0, 5) as usize];
        let options = PackOptions {
            format: if zip { "zip" } else { "7z" }.into(),
            level: [0, 1, 3, 5, 7, 9][w.get_level_index().clamp(0, 5) as usize],
            method: method.into(),
            dictionary: (!dict.is_empty() && method.starts_with("LZMA")).then(|| dict.into()),
            solid: ["off", "16m", "64m", "256m", "1g", "on"]
                [w.get_solid_index().clamp(0, 5) as usize]
                .into(),
            threads: ["on", "1", "2", "4", "8", "16", "32"]
                [w.get_threads_index().clamp(0, 6) as usize]
                .into(),
            volume: w.get_volume().into(),
            password: w.get_password().into(),
            encrypt_headers: w.get_encrypt_headers(),
            advanced: w.get_advanced().into(),
        };
        if let Err(error) = options.validate() {
            w.set_error(error.to_string().into());
            return;
        }
        // The dialog owns the only UI copy; never retain a password there.
        w.set_password("".into());
        let _ = w.hide();
        submit(Operation::Create {
            destination,
            inputs: inputs.clone(),
            options,
        });
    });
    ui.show()?;
    Ok(ui)
}
