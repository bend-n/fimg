use std::{marker::PhantomData, num::NonZeroU32};

use crate::Image;

/// A smaller part of a larger image.
///
/// ```text
/// ┏━━━━━━━━━━━━━━┓ hard borders represent the full image
/// ┃ 1   2  3   1 ┃                 vvvv the top left of the new image
/// ┃   ┌──────┐   ┃ crop(2, 2).from(1, 1)
/// ┃ 4 │ 5  6 │ 2 ┃      ^^^^ width and height
/// ┃   │      │   ┃
/// ┃ 7 │ 8  9 │ 3 ┃
/// ┗━━━┷━━━━━━┷━━━┛ soft borders represent the new image
/// ```
#[derive(Clone)]
pub struct SubImage<T, const CHANNELS: usize> {
    inner: Image<T, CHANNELS>,
    /// in pixels
    offset_x: u32,
    real_width: NonZeroU32,
    real_height: NonZeroU32,
}

/// Trait for cropping a image.
pub trait Cropper<T, const C: usize> {
    /// # Panics
    ///
    /// if w - y == 0
    fn from(self, x: u32, y: u32) -> SubImage<T, C>;
}

impl<T: Clone, const N: usize> Copy for SubImage<T, N> where Image<T, N>: Copy {}

macro_rules! def {
    ($t:ty, $($what:ident)?) => {
        struct Crop<'a, T, const C: usize> {
            dimensions: (NonZeroU32, NonZeroU32),
            _d: PhantomData<SubImage<$t, C>>,
            image: Image<$t, C>,
        }

        impl<'a, T, const C: usize> Cropper<$t, C> for Crop<'a, T, C> {
            fn from(self, x: u32, y: u32) -> SubImage<$t, C> {
                let w = self.image.width();
                // SAFETY: ctor
                let i = unsafe {
                    Image::new(
                        self.image.width,
                        NonZeroU32::new(self.image.height() - y).unwrap(),
                        &$($what)?(self.image.take_buffer()[(y as usize * C) * w as usize..]),
                    )
                };
                SubImage {
                    offset_x: x,
                    inner: i,
                    real_width: self.dimensions.0,
                    real_height: self.dimensions.1,
                }
            }
        }
    };
}

impl<T, const C: usize> Image<T, C> {
    /// Crop a image.
    ///
    /// The signature looks something like: `i.crop(width, height).from(top_left_x, top_left_y)`, which gives you a <code>[SubImage]<&\[T\], _></code>
    ///
    /// If you want a owned image, `i.crop(w, h).from(x, y).own()` gets you a <code>[`Image`]<[Box]<\[T\], _>></code> back.
    ///
    /// ```
    /// # use fimg::{Image, Cropper};
    /// let mut i = Image::<_, 1>::build(4, 3).buf([
    ///    1, 2, 3, 1,
    ///    4, 5, 6, 2,
    ///    7, 8, 9, 3,
    /// ]);
    /// let c = i.crop(2, 2).from(1, 1);
    /// # unsafe {
    /// assert_eq!(c.pixel(0, 0), [5]);
    /// assert_eq!(c.pixel(1, 1), [9]);
    /// assert_eq!(
    ///   c.own().bytes(),
    ///   &[5, 6,
    ///     8, 9]
    /// );
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// if width == 0 || height == 0
    pub fn crop<'a, U: 'a>(&'a self, width: u32, height: u32) -> impl Cropper<&'a [U], C>
    where
        T: AsRef<[U]>,
    {
        def!(&'a [T],);
        Crop {
            dimensions: (
                NonZeroU32::new(width).expect("Image::crop panics when width == 0"),
                NonZeroU32::new(height).expect("Image::crop panics when height == 0"),
            ),
            _d: PhantomData,
            image: self.as_ref(),
        }
    }

    /// Like [`Image::crop`], but returns a mutable [`SubImage`].
    pub fn crop_mut<'a, U: 'a>(
        &'a mut self,
        width: u32,
        height: u32,
    ) -> impl Cropper<&'a mut [U], C>
    where
        T: AsMut<[U]> + AsRef<[U]>,
    {
        def!(&'a mut [T], mut);
        Crop {
            dimensions: (
                NonZeroU32::new(width).expect("Image::crop panics when width == 0"),
                NonZeroU32::new(height).expect("Image::crop panics when height == 0"),
            ),
            _d: PhantomData,
            image: self.as_mut(),
        }
    }
}

impl<T: Clone, const C: usize> SubImage<&[T], C> {
    /// Clones this [`SubImage`] into its own [`Image`]
    pub fn own(&self) -> Image<Box<[T]>, C> {
        let mut out =
            Vec::with_capacity(self.real_width.get() as usize * self.inner.height() as usize * C);
        for row in self
            .inner
            .buffer
            .chunks_exact(self.inner.width.get() as usize)
            .take(self.real_height.get() as usize)
        {
            out.extend_from_slice(
                &row[self.offset_x as usize
                    ..self.offset_x as usize + self.real_width.get() as usize],
            );
        }
        // SAFETY: ctor
        unsafe { Image::new(self.real_width, self.real_height, out.into()) }
    }
}

// TODO crop()
impl<W, const C: usize> SubImage<W, C> {
    /// Get a pixel.
    ///
    /// # Safety
    ///
    /// this pixel must be in bounds.
    pub unsafe fn pixel<U: Copy>(&self, x: u32, y: u32) -> [U; C]
    where
        W: AsRef<[U]>,
    {
        // note: if you get a pixel, in release mode, that is in bounds of the outer image, but not the sub image, that would be library-ub.
        debug_assert!(x < self.real_width.get());
        debug_assert!(y < self.real_height.get());
        // SAFETY: caller
        unsafe { self.inner.pixel(x + self.offset_x, y) }
    }

    /// Get a pixel, mutably.
    ///
    /// # Safety
    ///
    /// this pixel must be in bounds.
    pub unsafe fn pixel_mut<U: Copy>(&mut self, x: u32, y: u32) -> &mut [U]
    where
        W: AsMut<[U]> + AsRef<[U]>,
    {
        debug_assert!(x < self.real_width.get());
        debug_assert!(y < self.real_height.get());
        // SAFETY: caller
        unsafe { self.inner.pixel_mut(x, y) }
    }
}
