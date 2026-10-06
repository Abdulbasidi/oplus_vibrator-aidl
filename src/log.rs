

use core::ffi::c_char;
use std::ffi::CString;

extern "C" {
    fn __android_log_write(prio: i32, tag: *const c_char, msg: *const c_char) -> i32;
}

pub fn log_line(prio: i32, msg: &str) {
    const TAG: &[u8] = b"hoshikv-vibrator-rs\0";
    let tag = TAG.as_ptr() as *const c_char;
    if let Ok(c) = CString::new(msg) {
        unsafe {
            __android_log_write(prio, tag, c.as_ptr());
        }
    }
}

#[macro_export]
macro_rules! alogi {
    ($($arg:tt)*) => {
        $crate::log::log_line(4, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! alogd {
    ($($arg:tt)*) => {
        $crate::log::log_line(3, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! aloge {
    ($($arg:tt)*) => {
        $crate::log::log_line(6, &format!($($arg)*))
    };
}