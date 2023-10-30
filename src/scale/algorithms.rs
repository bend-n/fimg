use super::{traits::*, *};
use std::num::NonZeroU32;

/// [Nearest Neighbor](https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation) image scaling algorithm.
pub struct Nearest;

impl ScalingAlgorithm for Nearest {
    /// Can be used on non opaque too! (Nearest is special like that).
    fn scale_opaque<const N: usize>(
        i: Image<&[u8], N>,
        w: NonZeroU32,
        h: NonZeroU32,
    ) -> Image<std::boxed::Box<[u8]>, N>
    where
        ChannelCount<N>: ToImageView<N>,
    {
        let mut dst = fr::Image::new(w, h);
        // SAFETY: swear, the pixel types are the same
        unsafe {
            fr::Resizer::new(fr::ResizeAlg::Nearest)
                .resize(&ChannelCount::<N>::wrap(i), &mut dst.view_mut())
        };

        // SAFETY: ctor
        unsafe { Image::new(dst.width(), dst.height(), dst.into_vec().into()) }
    }

    #[inline]
    fn scale_transparent<const N: usize>(
        i: Image<&mut [u8], N>,
        w: NonZeroU32,
        h: NonZeroU32,
    ) -> Image<std::boxed::Box<[u8]>, N>
    where
        ChannelCount<N>: AlphaDiv<N>,
    {
        Self::scale_opaque(i.as_ref(), w, h)
    }
}

macro_rules! alg {
    ($for:ident) => {
        impl ScalingAlgorithm for $for {
            fn scale_opaque<const N: usize>(
                i: Image<&[u8], N>,
                w: NonZeroU32,
                h: NonZeroU32,
            ) -> Image<std::boxed::Box<[u8]>, N>
            where
                ChannelCount<N>: ToImageView<N>,
            {
                let mut dst = fr::Image::new(w, h);
                // SAFETY: swear, the pixel types are the same
                unsafe {
                    fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::$for))
                        .resize(&ChannelCount::<N>::wrap(i), &mut dst.view_mut())
                };

                // SAFETY: ctor
                unsafe { Image::new(dst.width(), dst.height(), dst.into_vec().into()) }
            }

            fn scale_transparent<const N: usize>(
                i: Image<&mut [u8], N>,
                w: NonZeroU32,
                h: NonZeroU32,
            ) -> Image<std::boxed::Box<[u8]>, N>
            where
                ChannelCount<N>: AlphaDiv<N>,
            {
                let mut dst = fr::Image::new(w, h);
                // SAFETY: yes
                unsafe {
                    fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::$for))
                        .resize(&ChannelCount::<N>::handle(i).view(), &mut dst.view_mut())
                }

                ChannelCount::<N>::unhandle(&mut dst);

                // SAFETY: ctor
                unsafe { Image::new(dst.width(), dst.height(), dst.into_vec().into()) }
            }
        }
    };
}

/// [Lanczos](https://en.wikipedia.org/wiki/Lanczos_resampling) scaling with a filter size (*a*) of 3.
pub struct Lanczos3 {}
alg!(Lanczos3);

/// [Catmull-Rom](https://en.wikipedia.org/wiki/Centripetal_Catmull%E2%80%93Rom_spline) bicubic filtering.
pub struct CatmullRom {}
alg!(CatmullRom);

/// Linear interpolation.
pub struct Bilinear {}
alg!(Bilinear);

/// The opposite of [`Nearest`].
pub struct Box {}
alg!(Box);

/// Hamming filtering has the same performance as a [`Bilinear`] filter, while
/// providing image (downscaling) quality comparable to bicubic filters like
/// [`CatmullRom`] or [`Mitchell`]. Creates a sharper image than [`Bilinear`] filtering,
/// and doesn't have dislocations on local level like [`Box`] suffers from.
/// Not recommended for upscaling.
pub struct Hamming {}
alg!(Hamming);

/// [Mitchell–Netravali](https://en.wikipedia.org/wiki/Mitchell%E2%80%93Netravali_filters) bicubic filtering.
pub struct Mitchell {}
alg!(Mitchell);
