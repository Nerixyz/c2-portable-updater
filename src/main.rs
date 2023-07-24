#![deny(clippy::cargo)]

use extract::extract;
use result::ResultExt;

use args::Args;
use std::{ffi::OsString, io};
use widestring::U16CString;
use win::spawn_detatched;

mod args;
mod extract;
mod result;
mod win;

#[derive(Debug, thiserror::Error)]
enum SwitchToCurrentDirError {
    #[error("Failed to get the name of the current executable ({0})")]
    CannotGetExeName(io::Error),
    #[error("The updater is located at the root of the filesystem.")]
    ExeAtTopLevel,
    #[error("Failed to switch the working directory ({0})")]
    SwitchDir(io::Error),
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to start Chatterino ({0})")]
struct StartProcessError(io::Error);

fn switch_to_exe_dir() -> Result<OsString, SwitchToCurrentDirError> {
    let mut exe_path =
        std::env::current_exe().map_err(SwitchToCurrentDirError::CannotGetExeName)?;
    // directory of executable
    exe_path.pop();
    let directory = exe_path
        .file_name()
        .ok_or(SwitchToCurrentDirError::ExeAtTopLevel)?
        .to_owned();
    // directory of portable installation
    if !exe_path.pop() {
        return Err(SwitchToCurrentDirError::ExeAtTopLevel);
    }

    std::env::set_current_dir(exe_path).map_err(SwitchToCurrentDirError::SwitchDir)?;

    Ok(directory)
}

fn main() {
    let args = Args::read().error_and_exit();
    let updater_dir = switch_to_exe_dir().error_and_exit();
    extract(extract::Args {
        zip_path: &args.zip_path,
        updater_dir: &updater_dir,
    })
    .error_and_exit();

    if args.restart {
        let mut exe = std::env::current_dir().error_and_exit();
        exe.push("chatterino.exe");
        spawn_detatched(&U16CString::from_os_str(exe.as_os_str()).unwrap_or_default())
            .error_and_exit();
    }
}
