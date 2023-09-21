//! `Box<cat>`
use crate::Image;

impl<const CHANNELS: usize> Image<&mut [u8], CHANNELS> {
    /// Draw a bordered box
    ///
    /// # Safety
    ///
    /// UB if the box is out of bounds
    /// ```
    /// # use fimg::Image;
    /// let mut b = Image::alloc(10, 9);
    /// unsafe { b.as_mut().r#box((1, 1), 7, 6, [255]) };
    /// # assert_eq!(b.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")
    /// ```
    pub unsafe fn r#box(
        &mut self,
        (x1, y1): (u32, u32),
        width: u32,
        height: u32,
        c: [u8; CHANNELS],
    ) {
        // skip sides, leave that to second loop
        for x in x1 + 1..width + x1 {
            // top line
            // SAFETY: responsibility is on caller
            unsafe { self.set_pixel(x, x1, c) };
            // bottom line
            // SAFETY: shift responsibility
            unsafe { self.set_pixel(x, x1 + height, c) };
        }
        for y in y1..=height + y1 {
            // SAFETY: >> responsibility
            unsafe { self.set_pixel(y1, y, c) };
            // SAFETY: << responsibility
            unsafe { self.set_pixel(y1 + width, y, c) };
        }
    }

    /// Draw a *filled* box.
    ///
    /// # Safety
    ///
    /// UB if box is out of bounds
    /// ```
    /// # use fimg::Image;
    /// let mut b = Image::alloc(10, 9);
    /// unsafe { b.as_mut().filled_box((1, 1), 7, 6, [255]) };
    /// # assert_eq!(b.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")
    /// ```
    pub unsafe fn filled_box(
        &mut self,
        (x1, y1): (u32, u32),
        width: u32,
        height: u32,
        c: [u8; CHANNELS],
    ) {
        for x in x1..=width + x1 {
            for y in y1..=height + y1 {
                // SAFETY: fill it
                unsafe { self.set_pixel(x, y, c) };
            }
        }
    }
}
