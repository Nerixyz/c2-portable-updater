use std::{ffi::OsStr, io, path::Path, time::Duration};

use win_msgbox::AbortRetryIgnore;
use windows_sys::Win32::Foundation::ERROR_SHARING_VIOLATION;
use zip::read::ZipFile;

pub struct Args<'a> {
    pub zip_path: &'a OsStr,
    pub updater_dir: &'a OsStr,
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractError<'a> {
    #[error("Failed to open zip (path: {0:?}) - {1}.")]
    OpenZip(&'a OsStr, io::Error),
    #[error("Failed to read zip file ({0}).")]
    ReadZip(zip::result::ZipError),
    #[error("Failed to get a file by its index ({0}).")]
    NoZipFile(zip::result::ZipError),
    #[error("Encountered a bad filename in the zip file ({0}).")]
    BadFilePath(String),
    #[error("Installation was aborted. Consider installing Chatterino manually.")]
    Aborted,
}

#[derive(Debug, thiserror::Error)]
enum ProcessFileError {
    #[error("Failed to remove file to make room for directory ({0}).")]
    RemovingFileForDir(io::Error),
    #[error("Failed to create directory ({0}).")]
    CreateDir(io::Error),
    #[error("Failed to open file for writing ({0}).\nMake sure you closed all instances of Chatterino. If you're using the browser extension, you might need to close your browser too.")]
    OpenFile(io::Error),
    #[error("Failed to extract file ({0}).")]
    WriteFile(io::Error),
}

impl ProcessFileError {
    pub fn os_error(&self) -> Option<i32> {
        match self {
            ProcessFileError::RemovingFileForDir(e) => e.raw_os_error(),
            ProcessFileError::CreateDir(e) => e.raw_os_error(),
            ProcessFileError::OpenFile(e) => e.raw_os_error(),
            ProcessFileError::WriteFile(e) => e.raw_os_error(),
        }
    }
}

pub fn extract(
    Args {
        zip_path,
        updater_dir,
    }: Args,
) -> Result<bool, ExtractError> {
    let reader = std::fs::OpenOptions::new()
        .read(true)
        .open(zip_path)
        .map_err(|e| ExtractError::OpenZip(zip_path, e))?;

    let mut archive = zip::read::ZipArchive::new(reader).map_err(ExtractError::ReadZip)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(ExtractError::NoZipFile)?;
        let name = file
            .enclosed_name()
            .ok_or_else(|| ExtractError::BadFilePath(file.name().to_owned()))?;
        let stripped = strip_path(&name);

        if stripped.as_os_str().is_empty() || stripped.starts_with(updater_dir) {
            continue; // skip the updater
        }
        let stripped = stripped.to_owned();

        process_file_retrying(&mut file, &stripped)?;
    }

    Ok(true)
}

fn strip_path(path: &Path) -> &Path {
    path.strip_prefix("Chatterino2/").unwrap_or(path)
}

fn process_file_retrying<'a>(file: &mut ZipFile, path: &Path) -> Result<(), ExtractError<'a>> {
    loop {
        let Err(e) = process_file_backoff(file, path) else {
            break Ok(());
        };

        let error = format!("Failed to install '{}':\n{e}", path.to_string_lossy());
        match win_msgbox::warning::<AbortRetryIgnore>(&error)
            .title("Chatterino Updater")
            .show()
        {
            Ok(AbortRetryIgnore::Abort) => break Err(ExtractError::Aborted),
            Ok(AbortRetryIgnore::Retry) => continue,
            Ok(AbortRetryIgnore::Ignore) => break Ok(()),
            Err(e) => {
                eprintln!("MessageBox failed with {e:#x}");
                break Err(ExtractError::Aborted);
            }
        }
    }
}

/// Tries to extract a single file.
/// If the extraction fails with `ERROR_SHARING_VIOLATION` (32),
/// then the operation is retried after a backoff.
fn process_file_backoff(file: &mut ZipFile, path: &Path) -> Result<(), ProcessFileError> {
    let mut backoff = 1u64 << 8;
    loop {
        match process_file(file, path) {
            Ok(_) => break Ok(()),
            Err(e) if backoff <= 4096 && e.os_error() == Some(ERROR_SHARING_VIOLATION as i32) => {
                std::thread::sleep(Duration::from_millis(backoff));
                backoff <<= 1;
            }
            Err(e) => break Err(e),
        }
    }
}

/// Extracts the `file` to `path`.
fn process_file(file: &mut ZipFile, path: &Path) -> Result<(), ProcessFileError> {
    if file.is_dir() {
        if path.exists() {
            if path.is_dir() {
                return Ok(());
            }
            std::fs::remove_file(path).map_err(ProcessFileError::RemovingFileForDir)?;
        }
        std::fs::create_dir(path).map_err(ProcessFileError::CreateDir)?;
        return Ok(());
    }

    let mut writer = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .map_err(ProcessFileError::OpenFile)?;

    std::io::copy(file, &mut writer).map_err(ProcessFileError::WriteFile)?;

    Ok(())
}
