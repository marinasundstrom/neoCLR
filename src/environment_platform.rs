//! Execution-host platform discovery shared with the native host adapter.
use std::ffi::{c_char, c_int};

unsafe extern "C" {
    fn neoclr_platform_info(selector: c_int, output: *mut c_char, capacity: usize) -> c_int;
}

pub(crate) fn read(selector: i32) -> String {
    let mut bytes = [0u8; 4096];
    // The adapter writes at most capacity bytes, including its trailing NUL.
    let length = unsafe { neoclr_platform_info(selector, bytes.as_mut_ptr().cast(), bytes.len()) };
    if length < 0 || length as usize >= bytes.len() {
        return String::new();
    }
    String::from_utf8(bytes[..length as usize].to_vec()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::read;

    #[test]
    fn execution_host_facts_and_invalid_selector() {
        assert!(!read(0).is_empty());
        assert!(!read(4).is_empty());
        assert!(!read(5).is_empty());
        assert_eq!(read(6), if cfg!(windows) { "\r\n" } else { "\n" });
        assert_eq!(read(-1), "");
        #[cfg(target_os = "macos")]
        {
            assert_eq!(read(0), "macos");
            assert!(read(10).parse::<u32>().unwrap() >= 10);
        }
        #[cfg(target_os = "linux")]
        {
            assert_eq!(read(0), "linux");
            assert_eq!(
                read(1),
                std::fs::read_to_string("/proc/sys/kernel/osrelease")
                    .unwrap()
                    .trim()
            );
        }
        #[cfg(windows)]
        {
            assert_eq!(read(0), "windows");
            assert!(read(10).parse::<u32>().unwrap() >= 6);
        }
    }
}
