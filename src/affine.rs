use crate::{FromRefMut, Image};

pub trait Rotations {
    /// Rotate a image 180 degrees clockwise.
    fn rot_180(&mut self);
    /// Rotate a image 90 degrees clockwise.
    /// # Safety
    ///
    /// UB if the image is not square
    unsafe fn rot_90(&mut self);
    /// Rotate a image 270 degrees clockwise, or 90 degrees anti clockwise.
    /// # Safety
    ///
    /// UB if the image is not square
    unsafe fn rot_270(&mut self);
}

pub trait Flips {
    /// Flip a image vertically.
    fn flip_v(&mut self);

    /// Flip a image horizontally.
    fn flip_h(&mut self);
}

impl<const CHANNELS: usize> Flips for Image<Vec<u8>, CHANNELS> {
    fn flip_h(&mut self) {
        self.as_mut().flip_h();
    }
    fn flip_v(&mut self) {
        self.as_mut().flip_v();
    }
}

impl<const CHANNELS: usize> Flips for Image<&mut [u8], CHANNELS> {
    fn flip_v(&mut self) {
        for y in 0..self.height() / 2 {
            for x in 0..self.width() {
                let y2 = self.height() - y - 1;
                // SAFETY: within bounds
                let p2 = unsafe { self.pixel(x, y2) };
                let p = unsafe { self.pixel(x, y) };
                unsafe { self.set_pixel(x, y2, p) };
                unsafe { self.set_pixel(x, y, p2) };
            }
        }
    }

    fn flip_h(&mut self) {
        for y in 0..self.height() {
            for x in 0..self.width() / 2 {
                let x2 = self.width() - x - 1;
                let p2 = unsafe { self.pixel(x2, y) };
                let p = unsafe { self.pixel(x, y) };
                unsafe { self.set_pixel(x2, y, p) };
                unsafe { self.set_pixel(x, y, p2) };
            }
        }
    }
}

impl<const CHANNELS: usize> Rotations for Image<Vec<u8>, CHANNELS> {
    fn rot_180(&mut self) {
        self.as_mut().rot_180();
    }

    unsafe fn rot_90(&mut self) {
        unsafe { self.as_mut().rot_90() }
    }

    unsafe fn rot_270(&mut self) {
        unsafe { self.as_mut().rot_270() }
    }
}

impl<const CHANNELS: usize> Rotations for Image<&mut [u8], CHANNELS> {
    fn rot_180(&mut self) {
        for y in 0..self.height() / 2 {
            for x in 0..self.width() {
                let p = unsafe { self.pixel(x, y) };
                let x2 = self.width() - x - 1;
                let y2 = self.height() - y - 1;
                let p2 = unsafe { self.pixel(x2, y2) };
                unsafe { self.set_pixel(x, y, p2) };
                unsafe { self.set_pixel(x2, y2, p) };
            }
        }

        if self.height() % 2 != 0 {
            let middle = self.height() / 2;

            for x in 0..self.width() / 2 {
                let p = unsafe { self.pixel(x, middle) };
                let x2 = self.width() - x - 1;
                let p2 = unsafe { self.pixel(x2, middle) };
                unsafe { self.set_pixel(x, middle, p2) };
                unsafe { self.set_pixel(x2, middle, p) };
            }
        }
    }

    #[inline]
    unsafe fn rot_90(&mut self) {
        // This is done by first flipping
        self.flip_v();
        // Then transposing the image, to save allocations.
        // SAFETY: caller ensures square
        unsafe { transpose(self) };
    }

    #[inline]
    unsafe fn rot_270(&mut self) {
        self.flip_h();
        // SAFETY: caller ensures squareness
        unsafe { transpose(self) };
    }
}

/// Transpose a square image
/// # Safety
///
/// UB if supplied image rectangular
unsafe fn transpose<const CHANNELS: usize>(img: &mut Image<&mut [u8], CHANNELS>) {
    debug_assert_eq!(img.width(), img.height());
    let size = img.width() as usize;
    // SAFETY: no half pixels
    let b = unsafe { img.buffer.as_chunks_unchecked_mut::<CHANNELS>() };
    for i in 0..size {
        for j in i..size {
            // SAFETY: caller ensures squarity
            unsafe {
                b.swap_unchecked(i * size + j, j * size + i);
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::img;

    #[test]
    fn rotate_90() {
        let mut from = img![
            [00, 01]
            [02, 10]
        ];
        unsafe { from.rot_90() };
        assert_eq!(
            from,
            img![
                [02, 00]
                [10, 01]
            ]
        );
    }

    #[test]
    fn rotate_180() {
        let mut from = img![
            [00, 01]
            [02, 10]
        ];
        from.rot_180();
        assert_eq!(
            from,
            img![
                [10, 02]
                [01, 00]
            ]
        );
    }

    #[test]
    fn rotate_270() {
        let mut from = img![
            [00, 01]
            [20, 10]
        ];
        unsafe { from.rot_270() };
        assert_eq!(
            from,
            img![
                [01, 10]
                [00, 20]
            ]
        );
    }

    #[test]
    fn flip_vertical() {
        let mut from = img![
            [90, 01]
            [21, 42]
        ];
        from.flip_v();
        assert_eq!(
            from,
            img![
                [21, 42]
                [90, 01]
            ]
        );
    }
    #[test]
    fn flip_horizontal() {
        let mut from = img![
            [90, 01]
            [21, 42]
        ];
        from.flip_h();
        assert_eq!(
            from,
            img![
                [01, 90]
                [42, 21]
            ]
        );
    }
}
