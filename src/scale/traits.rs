//! implementation detail for scaling. look into if you want to add a algorithm
use std::num::NonZeroU32;

#[doc(hidden)]
mod seal {
    #[doc(hidden)]
    pub trait Sealed {}
}

use seal::Sealed;

use crate::Image;
impl Sealed for ChannelCount<1> {}
impl Sealed for ChannelCount<2> {}
impl Sealed for ChannelCount<3> {}
impl Sealed for ChannelCount<4> {}

/// How to scale a image
pub trait ScalingAlgorithm {
    /// Y/Rgb scale
    fn scale_opaque<const N: usize>(
        i: Image<&[u8], N>,
        w: NonZeroU32,
        h: NonZeroU32,
    ) -> Image<Box<[u8]>, N>
    where
        ChannelCount<N>: ToImageView<N>;
    /// Ya/Rgba scale
    fn scale_transparent<const N: usize>(
        i: Image<&mut [u8], N>,
        w: NonZeroU32,
        h: NonZeroU32,
    ) -> Image<Box<[u8]>, N>
    where
        ChannelCount<N>: AlphaDiv<N>;
}

/// helper
pub trait ToImageView<const N: usize>: Sealed {
    #[doc(hidden)]
    type P: fr::PixelExt + fr::Convolution;
    #[doc(hidden)]
    fn wrap(i: Image<&[u8], N>) -> fr::ImageView<Self::P>;
}

/// helper
pub trait AlphaDiv<const N: usize>: Sealed + ToImageView<N> {
    #[doc(hidden)]
    type P: fr::PixelExt + fr::Convolution + fr::AlphaMulDiv;
    #[doc(hidden)]
    fn handle(i: Image<&mut [u8], N>) -> fr::Image<'_, <Self as AlphaDiv<N>>::P>;
}

/// Generic helper for [`Image`] and [`fr::Image`] transfers.
pub struct ChannelCount<const N: usize> {}

macro_rules! tiv {
    ($n:literal, $which:ident) => {
        impl ToImageView<$n> for ChannelCount<$n> {
            type P = fr::$which;
            fn wrap(i: Image<&[u8], $n>) -> fr::ImageView<Self::P> {
                // SAFETY: same conds
                unsafe { fr::ImageView::new(i.width, i.height, i.buffer()) }
            }
        }
    };
}

tiv!(1, U8);
tiv!(2, U8x2);
tiv!(3, U8x3);
tiv!(4, U8x4);

macro_rules! adiv {
    ($n:literal, $which:ident) => {
        impl AlphaDiv<$n> for ChannelCount<$n> {
            type P = fr::$which;
            fn handle(i: Image<&mut [u8], $n>) -> fr::Image<<Self as AlphaDiv<$n>>::P> {
                // SAFETY: we kinda have the same conditions
                let mut i = unsafe { fr::Image::from_slice_u8(i.width, i.height, i.take_buffer()) };
                // SAFETY: mhm
                unsafe { fr::MulDiv::default().multiply_alpha_inplace(&mut i.view_mut()) };

                i
            }
        }
    };
}

adiv!(2, U8x2);
adiv!(4, U8x4);
