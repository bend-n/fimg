#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;
use std::cmp::max;

#[cfg(unix)]
pub use unix::size;
#[cfg(windows)]
pub use windows::size;
#[cfg(all(not(unix), not(windows)))]
pub fn size() -> Option<(u16, u16)> {
    #[cfg(debug_assertions)]
    eprintln!("unable to get terminal size");
    None
}

pub fn fit((w, h): (u32, u32)) -> (u32, u32) {
    if let Some((mw, mh)) = size().map(|(a, b)| (a as u32, b as u32)) {
        match () {
            () if w <= mw && h <= 2 * mh => (w, 2 * max(1, h / 2 + h % 2) - h % 2),
            () if mw * h <= w * 2 * mh => (mw, 2 * max(1, h * mw / w / 2) - h % 2),
            () => (w * 2 * mh / h, 2 * max(1, 2 * mh / 2) - h % 2),
        }
    } else {
        (w, h)
    }
}
