use std::path::PathBuf;

pub const ENGINE_NAME: &str = "7zz.exe";
pub fn quiet_command(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000);
}

/// The settings switch, named after this system's file manager.
pub const SHELL_MENU_LABEL: &str = "Show in the Explorer context menu";

/// Explorer, through the per-user registry hive.
pub const SHELL_MENU: bool = true;

/// `%APPDATA%\HatchZip`, which roams with the user's profile.
pub fn config_dir() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA").filter(|v| !v.is_empty())?;
    Some(PathBuf::from(appdata).join("7zip-hatch"))
}

pub fn install_open_documents(_drops: std::rc::Rc<crate::backend::Drops>) {}
