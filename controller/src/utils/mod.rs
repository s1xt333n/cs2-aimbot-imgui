use std::ffi::CString;

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HWND,
        UI::{
            Shell::ShellExecuteA,
            WindowsAndMessaging::SW_SHOW,
        },
    },
};

pub mod neuomorphic;
// pub use neuomorphic::*; // Temporarily commented out

pub mod ragnarek;
// pub use ragnarek::*; // Temporarily commented out

mod console_io;
pub use console_io::*;

mod imgui;
pub use self::imgui::*;

#[allow(unused)]
pub fn open_url(url: &str) {
    unsafe {
        let url = match CString::new(url) {
            Ok(url) => url,
            Err(_) => return,
        };

        ShellExecuteA(
            HWND::default(),
            PCSTR::null(),
            PCSTR(url.as_bytes().as_ptr()),
            PCSTR::null(),
            PCSTR::null(),
            SW_SHOW,
        );
    }
}
