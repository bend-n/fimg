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

impl Nearest {
    /// Resize a image.
    /// # Safety
    ///
    /// `image` must be as big or bigger than `width`, `height.
    #[must_use = "function does not modify the original image"]
    #[deprecated = "use Image::scale instead (note that Image::scale does not support any N. if there is a N you would like to see supported, please open a issue)"]
    pub unsafe fn scale<const N: usize>(
        image: Image<&[u8], N>,
        width: u32,
        height: u32,
    ) -> Image<Vec<u8>, N> {
        let x_scale = image.width() as f32 / width as f32;
        let y_scale = image.height() as f32 / height as f32;
        let mut out = Image::alloc(width, height);
        for y in 0..height {
            for x in 0..width {
                let x1 = ((x as f32 + 0.5) * x_scale).floor() as u32;
                let y1 = ((y as f32 + 0.5) * y_scale).floor() as u32;
                // SAFETY: i asked the caller to make sure its ok
                let px = unsafe { image.pixel(x1, y1) };
                // SAFETY: were looping over the width and height of out. its ok.
                unsafe { out.set_pixel(x, y, px) };
            }
        }
        out
    }
}
