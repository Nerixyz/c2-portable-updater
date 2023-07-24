use std::ptr;
use widestring::U16CStr;
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, FALSE, TRUE},
    System::{
        Diagnostics::Debug::{
            FormatMessageA, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
        },
        Threading::{
            CreateProcessW, CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS, PROCESS_INFORMATION,
            STARTUPINFOW,
        },
    },
};

pub fn spawn_detatched(path: &U16CStr) -> Result<(), String> {
    unsafe {
        let mut si: STARTUPINFOW = std::mem::zeroed();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();

        if CreateProcessW(
            path.as_ptr(),
            ptr::null_mut(), // no command line arguments
            ptr::null(),     // their process handle isn't inheritable
            ptr::null(),     // their thread handle isn't inheritable
            FALSE,           // our handles aren't inheritable
            DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
            ptr::null(), // inherit environment
            ptr::null(), // use our CWD
            &si,
            &mut pi,
        ) != TRUE
        {
            return Err(format_win_error("Failed to spawn process", GetLastError()));
        }

        // the process was spawned detached - we're done with it
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);

        Ok(())
    }
}

fn format_win_error(op: &str, e: windows_sys::Win32::Foundation::WIN32_ERROR) -> String {
    let mut buf = [0u8; 256];
    let len = unsafe {
        FormatMessageA(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            std::ptr::null(),
            e,
            0,
            buf.as_mut_ptr(),
            buf.len() as u32,
            std::ptr::null(),
        ) as usize
    };
    let msg = std::str::from_utf8(&buf[..len]).unwrap_or_default(); // default: empty string

    format!("{op}: {msg} (0x{e:x})")
}

#[cfg(all(test, windows))]
mod tests {
    use windows_sys::Win32::Foundation::ERROR_SHARING_VIOLATION;

    use super::*;

    #[test]
    fn error_fmt() {
        let fmt = format_win_error("context", ERROR_SHARING_VIOLATION);
        // we don't know the exact message, but thre must be at least a character
        assert!(
            !fmt.strip_prefix("context: ")
                .and_then(|s| s.strip_suffix(" (0x20)"))
                .map(|s| s.trim().is_empty())
                .unwrap_or(true),
            "{}",
            fmt
        );
    }
}
