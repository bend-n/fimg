use stackblur_iter::imgref::ImgRefMut;

use crate::{pixels::convert::PFrom, Image};

impl<T: AsMut<[u32]> + AsRef<[u32]>> Image<T, 1> {
    /// Blur a image of packed 32 bit integers, `[0xAARRGGBB]`.
    pub fn blur_argb(&mut self, radius: usize) {
        let w = self.width() as usize;
        let h = self.height() as usize;
        stackblur_iter::simd_blur_argb::<4>(&mut ImgRefMut::new(self.buffer.as_mut(), w, h), radius)
    }
}

impl<const N: usize> Image<Box<[u8]>, N>
where
    [u8; 4]: PFrom<N>,
    [u8; N]: PFrom<4>,
{
    /// Blur a image.
    /// ```
    /// # use fimg::Image;
    /// let mut i = Image::alloc(300, 300).boxed();
    /// // draw a lil pentagon
    /// i.poly((150., 150.), 5, 100.0, 0.0, [255]);
    /// // give it some blur
    /// i.blur(25);
    /// assert_eq!(include_bytes!("../tdata/blurred_pentagon.imgbuf"), i.bytes())
    /// ```
    pub fn blur(&mut self, radius: usize) {
        // you know, i optimized blurslice a fair bit, and yet, despite all the extra bit twiddling stackblur-iter is faster.
        let mut argb = Image::<Box<[u32]>, 1>::from(self.as_ref());
        argb.blur_argb(radius);
        for (i, n) in crate::convert::unpack_all(&argb.buffer).enumerate() {
            *unsafe { self.buffer.get_unchecked_mut(i) } = n;
        }
    }
}

impl<const N: usize> Image<&[u8], N>
where
    [u8; 4]: PFrom<N>,
    [u8; N]: PFrom<4>,
{
    /// Blur a image.
    pub fn blur(self, radius: usize) -> Image<Box<[u8]>, N> {
        let mut argb = Image::<Box<[u32]>, 1>::from(self);
        argb.blur_argb(radius);
        // SAFETY: ctor
        unsafe { Image::new(argb.width, argb.height, &**argb.buffer()) }.into()
    }
}
