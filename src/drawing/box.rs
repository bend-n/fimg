//! `Box<cat>`
use std::ops::{DerefMut, Range};

use crate::Image;

impl<T: DerefMut<Target = [u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// Draw a bordered box
    /// ```
    /// # use fimg::Image;
    /// let mut b = Image::alloc(10, 9);
    /// b.as_mut().r#box((1, 1), 7, 6, [255]);
    /// # assert_eq!(b.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")
    /// ```
    pub fn r#box(&mut self, (x1, y1): (u32, u32), width: u32, height: u32, c: [u8; CHANNELS]) {
        // skip sides, leave that to second loop
        for x in clamp(x1 + 1..width + x1, 0..self.width()) {
            // top line
            // SAFETY: clamped to bounds
            unsafe { self.set_pixel(x, x1, c) };
            // SAFETY: clamped to bounds
            unsafe { self.set_pixel(x, x1 + height, c) };
        }
        for y in clamp(y1..height + y1 + 1, 0..self.height()) {
            // SAFETY: clamped to bounds
            unsafe { self.set_pixel(y1, y, c) };
            // SAFETY: clamped to bounds
            unsafe { self.set_pixel(y1 + width, y, c) };
        }
    }

    /// Draw a *filled* box.
    /// ```
    /// # use fimg::Image;
    /// let mut b = Image::alloc(10, 9);
    /// b.as_mut().filled_box((1, 1), 7, 6, [255]);
    /// # assert_eq!(b.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")
    /// ```
    pub fn filled_box(&mut self, (x1, y1): (u32, u32), width: u32, height: u32, c: [u8; CHANNELS]) {
        for x in clamp(x1..1 + width + x1, 0..self.width()) {
            for y in clamp(y1..1 + height + y1, 0..self.height()) {
                // SAFETY: clamped to bounds
                unsafe { self.set_pixel(x, y, c) };
            }
        }
    }
}

/// clamp a range with another range
fn clamp(r: Range<u32>, within: Range<u32>) -> Range<u32> {
    r.start.clamp(within.start, within.end)..r.end.clamp(within.start, within.end)
}
