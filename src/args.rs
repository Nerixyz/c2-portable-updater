use std::ffi::{OsStr, OsString};

pub struct Args {
    pub zip_path: OsString,
    pub restart: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, thiserror::Error)]
pub enum ReadArgsError {
    #[error("The updater cannot be ran manually - No arguments were provided.")]
    NoArgs,
}

impl Args {
    pub fn read() -> Result<Self, ReadArgsError> {
        let mut it = std::env::args_os().skip(1);
        let zip_path = it.next().ok_or(ReadArgsError::NoArgs)?;
        let restart = it.any(|arg| arg == OsStr::new("restart"));

        Ok(Self { zip_path, restart })
    }
}
