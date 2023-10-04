//! Manages the affine image transformations.
use crate::{cloner::ImageCloner, Image};

impl<const CHANNELS: usize> ImageCloner<'_, CHANNELS> {
    /// Flip an image vertically.
    /// ```
    /// # use fimg::Image;
    /// let a = Image::<_, 1>::build(2,2).buf(vec![21,42,90,01]);
    /// assert_eq!(a.cloner().flip_v().take_buffer(), [90,01,21,42]);
    /// ```
    #[must_use = "function does not modify the original image"]
    pub fn flip_v(&self) -> Image<Vec<u8>, CHANNELS> {
        let mut out = self.alloc();
        for y in 0..self.height() {
            for x in 0..self.width() {
                // SAFETY: looping over self, all ok (could be safe versions, bounds would be elided)
                let p = unsafe { self.pixel(x, y) };
                // SAFETY: looping over self.
                unsafe { out.set_pixel(x, self.height() - y - 1, p) };
            }
        }
        out
    }

    /// Flip an image horizontally
    /// ```
    /// # use fimg::Image;
    /// let a = Image::<_,1>::build(2,2).buf(vec![90,01,21,42]);
    /// assert_eq!(a.cloner().flip_h().take_buffer(), [01,90,42,21]);
    /// ```
    #[must_use = "function does not modify the original image"]
    pub fn flip_h(&self) -> Image<Vec<u8>, CHANNELS> {
        let mut out = self.alloc();
        for y in 0..self.height() {
            for x in 0..self.width() {
                // SAFETY: looping over self, all ok
                let p = unsafe { self.pixel(x, y) };
                // SAFETY: looping over self, all ok
                unsafe { out.set_pixel(self.width() - x - 1, y, p) };
            }
        }
        out
    }
}

impl<const CHANNELS: usize, T: AsMut<[u8]> + AsRef<[u8]>> Image<T, CHANNELS> {
    /// Flip an image vertically.
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

    /// Flip an image horizontally.
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

impl<const CHANNELS: usize> ImageCloner<'_, CHANNELS> {
    /// Rotate an image 180 degrees clockwise.
    ///
    /// ```
    /// # use fimg::Image;
    /// let a = Image::<_,1>::build(2,2).buf(vec![00,01,02,10]);
    /// assert_eq!(a.cloner().rot_180().take_buffer(), vec![10,02,01,00]);
    /// ```
    #[must_use = "function does not modify the original image"]
    pub fn rot_180(&self) -> Image<Vec<u8>, CHANNELS> {
        let s = (self.width() * self.height()) as usize;
        let mut v: Vec<[u8; CHANNELS]> = Vec::with_capacity(s);
        for (x, y) in self.chunked().rev().zip(&mut v.spare_capacity_mut()[..]) {
            y.write(*x);
        }
        // SAFETY: we just wrote the right amount
        unsafe { v.set_len(s) };
        Image::build(self.width(), self.height()).buf(v.into_flattened())
    }

    /// Rotate an image 90 degrees clockwise.
    /// # Safety
    ///
    /// UB if the image is not square
    #[must_use = "function does not modify the original image"]
    pub unsafe fn rot_90(&self) -> Image<Vec<u8>, CHANNELS> {
        // SAFETY: yep
        let mut out = unsafe { transpose_out(self) };
        // SAFETY: sqar
        unsafe { crev(out.as_mut()) };
        out
    }

    /// Rotate an image 270 degrees clockwise, or 90 degrees anti clockwise.
    /// # Safety
    ///
    /// UB if the image is not square
    #[must_use = "function does not modify the original image"]
    pub unsafe fn rot_270(&self) -> Image<Vec<u8>, CHANNELS> {
        // SAFETY: yep
        let mut out = unsafe { transpose_out(self) };
        out.flip_v();
        out
    }
}

impl<const CHANNELS: usize, T: AsMut<[u8]> + AsRef<[u8]>> Image<T, CHANNELS> {
    /// Rotate an image 180 degrees clockwise.
    pub fn rot_180(&mut self) {
        self.flatten_mut().reverse();
    }

    /// Rotate an image 90 degrees clockwise.
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

    /// Rotate an image 270 degrees clockwise, or 90 degrees anti clockwise.
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

/// Reverse columns of square image
/// # Safety
///
/// UB if supplied image not square
unsafe fn crev<const CHANNELS: usize, T: AsMut<[u8]> + AsRef<[u8]>>(mut img: Image<T, CHANNELS>) {
    debug_assert_eq!(img.width(), img.height());
    let size = img.width() as usize;
    let b = img.flatten_mut();
    for i in 0..size {
        let mut start = 0;
        let mut end = size - 1;
        while start < end {
            // SAFETY: hmm
            unsafe { b.swap_unchecked(i * size + start, i * size + end) };
            start += 1;
            end -= 1;
        }
    }
}

/// Transpose a square image out of place
/// # Safety
///
/// UB if provided image rectangular
unsafe fn transpose_out<const CHANNELS: usize>(
    i: &ImageCloner<'_, CHANNELS>,
) -> Image<Vec<u8>, CHANNELS> {
    let mut out = i.alloc();
    // SAFETY: yep
    unsafe {
        mattr::transpose(
            i.flatten(),
            out.flatten_mut(),
            i.height() as usize,
            i.width() as usize,
        )
    };
    out
}

/// Transpose a square image
/// # Safety
///
/// UB if supplied image rectangular
unsafe fn transpose<const CHANNELS: usize, T: AsMut<[u8]> + AsRef<[u8]>>(
    img: &mut Image<T, CHANNELS>,
) {
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
unsafe fn transpose_non_power_of_two<const CHANNELS: usize, T: AsMut<[u8]> + AsRef<[u8]>>(
    img: &mut Image<T, CHANNELS>,
) {
    debug_assert_eq!(img.width(), img.height());
    let size = img.width() as usize;
    let b = img.flatten_mut();
    for i in 0..size {
        for j in i..size {
            // SAFETY: caller ensures squarity
            unsafe { b.swap_unchecked(i * size + j, j * size + i) };
        }
    }
}

/// break it down until
const TILE: usize = 4;
/// # Safety
///
/// be careful
unsafe fn transpose_tile<const CHANNELS: usize, T: AsMut<[u8]> + AsRef<[u8]>>(
    img: &mut Image<T, CHANNELS>,
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
unsafe fn transpose_diag<const CHANNELS: usize, T: AsMut<[u8]> + AsRef<[u8]>>(
    img: &mut Image<T, CHANNELS>,
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
