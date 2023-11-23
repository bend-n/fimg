use slur::{
    color::{u32xN, BlurU32},
    imgref::ImgRefMut,
};
use std::{
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    simd::Simd,
};

use crate::Image;

impl<T: AsMut<[u32]> + AsRef<[u32]>> Image<T, 1> {
    /// Blur a image of packed 32 bit integers, `[0xAARRGGBB]`.
    pub fn blur_argb(&mut self, radius: usize) {
        let w = self.width() as usize;
        let h = self.height() as usize;
        slur::simd_blur_argb::<4>(&mut ImgRefMut::new(self.buffer.as_mut(), w, h), radius)
    }
}

macro_rules! simd {
    ($n:literal) => {
        impl<T: AsMut<[u8]> + AsRef<[u8]>> Image<T, $n> {
            /// Blur a image.
            pub fn blur_in(&mut self, radius: usize) {
                let (w, h) = (self.width() as usize, self.height() as usize);
                let px = self.flatten_mut();
                slur::blur(
                    &mut ImgRefMut::new(px, w, h),
                    radius,
                    |x| slur::color::u32xN(std::simd::Simd::from_array(x.map(|x| x as u32))),
                    |x| x.0.to_array().map(|x| x as u8),
                );
            }
        }
    };
}

simd!(4);
simd!(2);

impl<T: AsMut<[u8]> + AsRef<[u8]>> Image<T, 3> {
    /// Blur a image.
    /// ```
    /// # use fimg::Image;
    /// let mut i = Image::alloc(300, 300).boxed();
    /// // draw a trongle
    /// i.poly((150., 150.), 3, 100.0, 0.0, [255, 255, 255]);
    /// // give it some blur
    /// i.blur_in(25);
    /// ```
    pub fn blur_in(&mut self, radius: usize) {
        let (w, h) = (self.width() as usize, self.height() as usize);
        let px = self.flatten_mut();
        slur::blur(
            &mut ImgRefMut::new(px, w, h),
            radius,
            |x| Px::from(*x),
            |x| x.into(),
        );
    }
}

impl<T: AsMut<[u8]> + AsRef<[u8]>> Image<T, 1> {
    /// Blur a image. No copy.
    /// ```
    /// # use fimg::Image;
    /// let mut i = Image::alloc(300, 300);
    /// // draw a lil pentagon
    /// i.poly((150., 150.), 5, 100.0, 0.0, [255]);
    /// // give it some blur
    /// i.blur(25);
    /// # assert_eq!(include_bytes!("../tdata/blurred_pentagon.imgbuf"), i.bytes())
    /// ```
    pub fn blur(&mut self, radius: usize) {
        let (w, h) = (self.width() as usize, self.height() as usize);
        slur::simd_blur::<_, _, _, 8>(
            &mut ImgRefMut::new(self.buffer.as_mut(), w, h),
            radius,
            |x| u32xN(Simd::from_array(x.map(|&x| x as u32))),
            |x| x.0.as_array().map(|x| x as u8),
            |&x| BlurU32(x as u32),
            |x| x.0 as u8,
        );
    }
}

macro_rules! blur_packing {
    ($n:literal) => {
        impl<T: AsRef<[u8]> + AsMut<[u8]>> Image<T, $n> {
            /// Blur a image. This will allocate a <code>[Image]<[Box]<[[u32]]>, 1></code>.
            /// If you want no copy, but slower if you dont have a simd-able cpu, check out [`Image::blur_in`].
            /// ```
            /// # use fimg::Image;
            /// let mut i = Image::alloc(300, 300);
            /// // draw a sqar
            /// i.poly((150., 150.), 4, 100.0, 0.0, [255]);
            /// // give it some blur
            /// i.blur(25);
            /// ```
            pub fn blur(&mut self, radius: usize) {
                // the bit twiddling lets it simd better
                let mut argb = Image::<Box<[u32]>, 1>::from(self.as_ref());
                argb.blur_argb(radius);
                for (i, n) in crate::convert::unpack_all::<$n>(&argb.buffer).enumerate() {
                    *unsafe { self.buffer.as_mut().get_unchecked_mut(i) } = n;
                }
            }
        }
    };
}
blur_packing!(2);
blur_packing!(3);
blur_packing!(4);

#[repr(transparent)]
#[derive(Copy, Clone)]
struct Px<const N: usize>([u32; N]);

impl<const N: usize> Default for Px<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> From<[u8; N]> for Px<N> {
    fn from(x: [u8; N]) -> Self {
        Self(x.map(|x| x as u32))
    }
}

impl<const N: usize> From<Px<N>> for [u8; N] {
    fn from(v: Px<N>) -> Self {
        v.0.map(|x| x as u8)
    }
}

macro_rules! op {
    ($name:ident, $as:ident, $fn:ident, $ass_fn:ident, $meth:ident) => {
        impl<const N: usize> $name<usize> for Px<N> {
            type Output = Px<N>;

            fn $fn(self, rhs: usize) -> Self::Output {
                Self(self.0.map(|x| x.$meth(rhs as u32)))
            }
        }

        impl<const N: usize> $name for Px<N> {
            type Output = Px<N>;
            fn $fn(self, rhs: Px<N>) -> Self::Output {
                let mut out = [0; N];
                for ((a, b), x) in self.0.iter().zip(rhs.0.iter()).zip(out.iter_mut()) {
                    *x = a.$meth(*b);
                }
                Self(out)
            }
        }

        impl<const N: usize> $as for Px<N> {
            fn $ass_fn(&mut self, rhs: Self) {
                *self = self.$fn(rhs);
            }
        }
    };
}
op!(Mul, MulAssign, mul, mul_assign, wrapping_mul);
op!(Sub, SubAssign, sub, sub_assign, wrapping_sub);
op!(Add, AddAssign, add, add_assign, wrapping_add);
op!(Div, DivAssign, div, div_assign, wrapping_div);
