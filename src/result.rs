use std::fmt::Display;

use win_msgbox::Okay;

pub trait ResultExt<T> {
    fn error_and_exit(self) -> T;
}

impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn error_and_exit(self) -> T {
        match self {
            Ok(r) => r,
            Err(e) => {
                win_msgbox::error::<Okay>(&e.to_string())
                    .title("Chatterino Updater")
                    .show()
                    .ok();
                std::process::exit(1);
            }
        }
    }
}
