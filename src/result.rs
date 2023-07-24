use std::fmt::Display;

use widestring::{U16CStr, U16CString};
use win_msgbox::Okay;
use windows_sys::w;

pub trait ResultExt<T> {
    fn error_and_exit(self) -> T;
}

impl<T, E: Display> ResultExt<T> for Result<T, E> {
    #[must_use]
    fn error_and_exit(self) -> T {
        match self {
            Ok(r) => r,
            Err(e) => {
                // Can't avoid creating a string here
                let s = U16CString::from_str_truncate(e.to_string());
                show_and_exit(&s);
            }
        }
    }
}

fn show_and_exit(s: &U16CStr) -> ! {
    win_msgbox::error::<Okay>(s.as_ptr())
        .title(w!("Chatterino Updater"))
        .show()
        .ok();
    std::process::exit(1);
}
