use std::mem::MaybeUninit as MU;
use windows::Win32::System::Console::{
    GetConsoleScreenBufferInfo, GetStdHandle, CONSOLE_SCREEN_BUFFER_INFO as winsize,
    SMALL_RECT as rect, STD_OUTPUT_HANDLE,
};

pub fn size() -> Option<(u16, u16)> {
    // SAFETY: SYS
    unsafe {
        let mut info = MU::uninit();
        GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE).ok()?, info.as_mut_ptr())
            .ok()?;
        let winsize {
            srWindow:
                rect {
                    Top,
                    Left,
                    Right,
                    Bottom,
                },
            ..
        } = info.assume_init();
        Some(((Bottom - Top - 1) as u16, (Right - Left - 1) as u16))
    }
}
