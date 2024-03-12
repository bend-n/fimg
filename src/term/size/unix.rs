extern crate libc;
use std::fs::File;
use std::mem::MaybeUninit as MU;
use std::ops::Deref;
use std::os::fd::IntoRawFd;

use libc::*;

struct Fd(i32, bool);
impl Drop for Fd {
    fn drop(&mut self) {
        if self.1 {
            unsafe { close(self.0) };
        }
    }
}
impl Fd {
    pub fn new(x: File) -> Self {
        Self(x.into_raw_fd(), true)
    }
}

impl From<i32> for Fd {
    fn from(value: i32) -> Self {
        Self(value, false)
    }
}

impl Deref for Fd {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn size() -> Option<(u16, u16)> {
    // SAFETY: SYS
    unsafe {
        let mut size = MU::<winsize>::uninit();

        if ioctl(
            *File::open("/dev/tty")
                .map(Fd::new)
                .unwrap_or(Fd::from(STDIN_FILENO)),
            TIOCGWINSZ.into(),
            size.as_mut_ptr(),
        ) != -1
        {
            let winsize { ws_col, ws_row, .. } = size.assume_init();
            return Some((ws_col as _, ws_row as _)).filter(|&(w, h)| w != 0 && h != 0);
        }
        tput("cols").and_then(|w| tput("lines").map(|h| (w, h)))
    }
}

pub fn tput(arg: &'static str) -> Option<u16> {
    let x = std::process::Command::new("tput").arg(arg).output().ok()?;
    String::from_utf8(x.stdout)
        .ok()
        .and_then(|x| x.parse::<u16>().ok())
}

#[test]
fn t() {
    println!("{:?}", size().unwrap());
}
