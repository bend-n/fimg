//! `Box<cat>`
use std::ops::Range;

use crate::Image;

impl<T: AsMut<[u8]> + AsRef<[u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
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
            // SAFETY: clamped to bounds
            unsafe { self.set_pixel(x, y1, c) };
        }
        for x in clamp(x1 + 1..width + x1, 0..self.width()) {
            // SAFETY: clamped to bounds
            unsafe { self.set_pixel(x, (y1 + height).min(self.height() - 1), c) };
        }
        for y in clamp(y1..height + y1 + 1, 0..self.height()) {
            // SAFETY: clamped to bounds
            unsafe { self.set_pixel(x1, y, c) };
        }
        for y in clamp(y1..height + y1 + 1, 0..self.height()) {
            // SAFETY: clamped to bounds
            unsafe { self.set_pixel((x1 + width).min(self.width() - 1), y, c) };
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

    /// Draw a stroked box
    /// ```
    /// # use fimg::Image;
    /// let mut b = Image::alloc(11, 11);
    /// b.as_mut().stroked_box((2, 2), 6, 6, 2, [255]);
    /// # assert_eq!(b.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\x00\x00\x00\x00\x00\xff\xff\x00\x00\xff\xff\x00\x00\x00\x00\x00\xff\xff\x00\x00\xff\xff\x00\x00\x00\x00\x00\xff\xff\x00\x00\xff\xff\x00\x00\x00\x00\x00\xff\xff\x00\x00\xff\xff\x00\x00\x00\x00\x00\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
    /// ```
    pub fn stroked_box(
        &mut self,
        (x1, y1): (u32, u32),
        width: u32,
        height: u32,
        stroke: u32,
        c: [u8; CHANNELS],
    ) {
        let n = stroke / 2;
        for n in 0..=n {
            // TODO this is slightly stupid
            // move it up and left, expand w, h
            self.r#box((x1 - n, y1 - n), width + n + n, height + n + n, c)
        }
    }
}

/// clamp a range with another range
fn clamp(r: Range<u32>, within: Range<u32>) -> Range<u32> {
    r.start.clamp(within.start, within.end)..r.end.clamp(within.start, within.end)
}

#[cfg(test)]
mod tests {
    use crate::Image;
    #[test]
    fn box_oob() {
        let mut i = Image::alloc(5, 5);
        i.r#box((7, 7), 5, 5, [255]);
        assert_eq!(i.buffer(), &[0u8; 5 * 5]);
    }

    #[test]
    fn partial_oob() {
        let mut i = Image::alloc(5, 5);
        i.r#box((2, 2), 2, 17, [255]);
        assert_eq!(i.buffer(),b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\x00\x00\xff\x00\xff\x00\x00\xff\xff\xff");
    }

    #[test]
    fn thick_box_oob() {
        let mut i = Image::alloc(5, 5);
        i.stroked_box((7, 7), 5, 5, 2, [255]);
        assert_eq!(i.buffer(), &[0u8; 5 * 5]);
    }

    #[test]
    fn thick_box_partial_oob() {
        let mut i = Image::alloc(15, 15);
        i.stroked_box((2, 2), 4, 17, 2, [255]);
        // ideally the bottom would have a 2 stroke line, alas tis difficult.
        assert_eq!(i.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00");
    }
}
