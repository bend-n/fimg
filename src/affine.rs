//! Manages the affine image transformations.
use crate::Image;

impl<const CHANNELS: usize> Image<Vec<u8>, CHANNELS> {
    /// Flip a image horizontally.
    pub fn flip_h(&mut self) {
        self.as_mut().flip_h();
    }
    /// Flip a image vertically.
    pub fn flip_v(&mut self) {
        self.as_mut().flip_v();
    }
}

impl<const CHANNELS: usize> Image<&mut [u8], CHANNELS> {
    /// Flip a image vertically.
    pub fn flip_v(&mut self) {
        for y in 0..self.height() / 2 {
            for x in 0..self.width() {
                let y2 = self.height() - y - 1;
                #[allow(clippy::multiple_unsafe_ops_per_block)]
                // SAFETY: within bounds
                unsafe {
                    let p2 = self.pixel(x, y2);
                    let p = self.pixel(x, y);
                    self.set_pixel(x, y2, p);
                    self.set_pixel(x, y, p2);
                }
            }
        }
    }

    /// Flip a image horizontally.
    pub fn flip_h(&mut self) {
        for y in 0..self.height() {
            for x in 0..self.width() / 2 {
                let x2 = self.width() - x - 1;
                #[allow(clippy::multiple_unsafe_ops_per_block)]
                // SAFETY: bounded
                unsafe {
                    let p2 = self.pixel(x2, y);
                    let p = self.pixel(x, y);
                    self.set_pixel(x2, y, p);
                    self.set_pixel(x, y, p2);
                }
            }
        }
    }
}

impl<const CHANNELS: usize> Image<Vec<u8>, CHANNELS> {
    /// Rotate a image 180 degrees clockwise.
    pub fn rot_180(&mut self) {
        self.as_mut().rot_180();
    }

    /// Rotate a image 90 degrees clockwise.
    /// # Safety
    ///
    /// UB if the image is not square
    pub unsafe fn rot_90(&mut self) {
        // SAFETY: make sure to keep the safety docs linked
        unsafe { self.as_mut().rot_90() }
    }

    /// Rotate a image 270 degrees clockwise, or 90 degrees anti clockwise.
    /// # Safety
    ///
    /// UB if the image is not square
    pub unsafe fn rot_270(&mut self) {
        // SAFETY: idk this is just a convenience impl
        unsafe { self.as_mut().rot_270() }
    }
}

impl<const CHANNELS: usize> Image<&mut [u8], CHANNELS> {
    /// Rotate a image 180 degrees clockwise.
    pub fn rot_180(&mut self) {
        for y in 0..self.height() / 2 {
            for x in 0..self.width() {
                // SAFETY: x, y come from the loop, must be ok
                let p = unsafe { self.pixel(x, y) };
                let x2 = self.width() - x - 1;
                let y2 = self.height() - y - 1;
                // SAFETY: values are good
                let p2 = unsafe { self.pixel(x2, y2) };
                // SAFETY: swapping would be cool, alas.
                unsafe { self.set_pixel(x, y, p2) };
                // SAFETY: although maybe i can cast it to a `[[u8; CHANNELS]]` and swap that 🤔
                unsafe { self.set_pixel(x2, y2, p) };
            }
        }

        if self.height() % 2 != 0 {
            let middle = self.height() / 2;

            for x in 0..self.width() / 2 {
                let x2 = self.width() - x - 1;
                #[allow(clippy::multiple_unsafe_ops_per_block)]
                // SAFETY: its just doing the swappy
                unsafe {
                    let p = self.pixel(x, middle);
                    let p2 = self.pixel(x2, middle);
                    self.set_pixel(x, middle, p2);
                    self.set_pixel(x2, middle, p);
                }
            }
        }
    }

    /// Rotate a image 90 degrees clockwise.
    /// # Safety
    ///
    /// UB if the image is not square
    #[inline]
    pub unsafe fn rot_90(&mut self) {
        // This is done by first flipping
        self.flip_v();
        // Then transposing the image, as to not allocate.
        // SAFETY: caller ensures square
        unsafe { transpose(self) };
    }

    /// Rotate a image 270 degrees clockwise, or 90 degrees anti clockwise.
    /// # Safety
    ///
    /// UB if the image is not square
    #[inline]
    pub unsafe fn rot_270(&mut self) {
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
    if img.width().is_power_of_two() {
        // SAFETY: caller gurantees
        unsafe { transpose_diag(img, 0, img.width() as usize) };
    } else {
        // SAFETY: caller gurantees
        unsafe { transpose_non_power_of_two(img) };
    }
}

/// Transpose a square (non power of two) image.
///
/// # Safety
///
/// UB if image not square
unsafe fn transpose_non_power_of_two<const CHANNELS: usize>(img: &mut Image<&mut [u8], CHANNELS>) {
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

/// break it down until
const TILE: usize = 4;
/// # Safety
///
/// be careful
unsafe fn transpose_tile<const CHANNELS: usize>(
    img: &mut Image<&mut [u8], CHANNELS>,
    row: usize,
    col: usize,
    size: usize,
) {
    if size > TILE {
        #[allow(
            clippy::multiple_unsafe_ops_per_block,
            clippy::undocumented_unsafe_blocks
        )]
        unsafe {
            // top left
            transpose_tile(img, row, col, size / 2);
            // top right
            transpose_tile(img, row, col + size / 2, size / 2);
            // bottom left
            transpose_tile(img, row + size / 2, col, size / 2);
            // bottom right
            transpose_tile(img, row + size / 2, col + size / 2, size / 2);
        }
    } else {
        let s = img.width() as usize;
        let b = img.flatten_mut();
        for i in 0..size {
            for j in 0..size {
                // SAFETY: this should be okay if we careful
                unsafe { b.swap_unchecked((row + i) * s + (col + j), (col + j) * s + (row + i)) };
            }
        }
    }
}

/// # Safety
///
/// be careful
unsafe fn transpose_diag<const CHANNELS: usize>(
    img: &mut Image<&mut [u8], CHANNELS>,
    pos: usize,
    size: usize,
) {
    if size > TILE {
        #[allow(
            clippy::multiple_unsafe_ops_per_block,
            clippy::undocumented_unsafe_blocks
        )]
        unsafe {
            transpose_diag(img, pos, size / 2);
            transpose_tile(img, pos, pos + size / 2, size / 2);
            transpose_diag(img, pos + size / 2, size / 2);
        }
    } else {
        let s = img.width() as usize;
        let b = img.flatten_mut();
        for i in 1..size {
            for j in 0..i {
                // SAFETY: this is fine unless pos / size is out of bounds, which it cant be
                unsafe { b.swap_unchecked((pos + i) * s + (pos + j), (pos + j) * s + (pos + i)) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::img;

    #[test]
    fn transp() {
        #[rustfmt::skip]
        let mut i = Image::<_, 1>::build(8, 8).buf(vec![
            0, 0, 1, 1, 0, 0, 1, 1,
            0, 1, 0, 1, 1, 0, 1, 1,
            0, 1, 1, 0, 1, 0, 1, 1,
            0, 1, 1, 1, 0, 0, 1, 1,
            0, 1, 1, 1, 1, 0, 1, 1,
            0, 1, 1, 1, 1, 0, 0, 1,
            0, 1, 1, 1, 1, 0, 1, 0,
            0, 0, 1, 1, 1, 0, 1, 1,
        ]);
        unsafe { transpose(&mut i.as_mut()) };
        #[rustfmt::skip]
        assert_eq!(i.take_buffer(), vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 1, 1, 1, 1, 1, 0,
            1, 0, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 1, 1, 1,
            0, 1, 1, 0, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0,
            1, 1, 1, 1, 1, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 0, 1
        ]);
    }

    #[test]
    fn transp9() {
        #[rustfmt::skip]
        let mut i = Image::<_, 1>::build(9, 9).buf(vec![
            0, 0, 1, 1, 0, 0, 1, 1, 0,
            0, 1, 0, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 0, 1, 0, 1, 1, 0,
            0, 1, 1, 1, 0, 0, 1, 1, 0,
            0, 1, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 1, 0, 0, 1, 1,
            0, 1, 1, 1, 1, 0, 1, 0, 1,
            0, 0, 1, 1, 1, 0, 1, 1, 0,
            1, 1, 1, 0, 1, 1, 0, 1, 0,
        ]);
        unsafe { transpose(&mut i.as_mut()) };
        #[rustfmt::skip]
        assert_eq!(i.take_buffer(), vec![
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 1, 1, 1, 1, 1, 1, 0, 1,
            1, 0, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 1, 1, 1, 0,
            0, 1, 1, 0, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 0, 1, 1, 0,
            1, 1, 1, 1, 1, 1, 0, 1, 1,
            0, 1, 0, 0, 1, 1, 1, 0, 0
        ]);
    }

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
