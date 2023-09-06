//! # fimg
//!
//! Provides fast image operations, such as rotation, flipping, and overlaying.
#![feature(
    slice_swap_unchecked,
    generic_const_exprs,
    slice_as_chunks,
    unchecked_math,
    portable_simd,
    const_option,
    array_chunks,
    test
)]
#![warn(
    clippy::missing_docs_in_private_items,
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_const_for_fn,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn,
    clippy::dbg_macro,
    missing_docs
)]
#![allow(clippy::zero_prefixed_literal, incomplete_features)]

use std::{num::NonZeroU32, slice::SliceIndex};

mod affine;
pub mod builder;
mod overlay;
pub use overlay::{Overlay, OverlayAt};

/// like assert!(), but causes undefined behaviour at runtime when the condition is not met.
///
/// # Safety
/// UB if condition is false.
macro_rules! assert_unchecked {
    ($cond:expr) => {{
        if !$cond {
            #[cfg(debug_assertions)]
            let _ = ::core::ptr::NonNull::<()>::dangling().as_ref(); // force unsafe wrapping block
            #[cfg(debug_assertions)]
            panic!("assertion failed: {} returned false", stringify!($cond));
            #[cfg(not(debug_assertions))]
            std::hint::unreachable_unchecked()
        }
    }};
}
use assert_unchecked;

impl Image<&[u8], 3> {
    /// Repeat self till it fills a new image of size x, y
    /// # Safety
    ///
    /// UB if self's width is not a multiple of x, or self's height is not a multiple of y
    pub unsafe fn repeated(&self, x: u32, y: u32) -> Image<Vec<u8>, 3> {
        let mut img = Image::alloc(x, y); // could probably optimize this a ton but eh
        for x in 0..(x / self.width()) {
            for y in 0..(y / self.height()) {
                let a: &mut Image<&mut [u8], 3> = &mut img.as_mut();
                // SAFETY: caller upholds
                unsafe { a.overlay_at(self, x * self.width(), y * self.height()) };
            }
        }
        img
    }
}

/// calculates a column major index, with unchecked math
#[inline]
unsafe fn really_unsafe_index(x: u32, y: u32, w: u32) -> usize {
    // y * w + x
    // SAFETY: FIXME make safe math
    let tmp = unsafe { (y as usize).unchecked_mul(w as usize) };
    // SAFETY: FIXME make safe math
    unsafe { tmp.unchecked_add(x as usize) }
}

/// A image with a variable number of channels, and a nonzero size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image<T, const CHANNELS: usize> {
    /// column order 2d slice/vec
    buffer: T,
    /// image horizontal size
    width: NonZeroU32,
    /// image vertical size
    height: NonZeroU32,
}

impl<T, const CHANNELS: usize> Image<T, CHANNELS> {
    #[inline]
    /// get the height as a [`u32`]
    pub fn height(&self) -> u32 {
        self.height.into()
    }

    #[inline]
    /// get the width as a [`u32`]
    pub fn width(&self) -> u32 {
        self.width.into()
    }

    #[inline]
    /// create a new image
    ///
    /// # Safety
    ///
    /// does not check that buffer.len() == w * h * C
    ///
    /// using this with invalid values may result in future UB
    pub const unsafe fn new(width: NonZeroU32, height: NonZeroU32, buffer: T) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    /// consumes the image, returning the image buffer
    pub fn take_buffer(self) -> T {
        self.buffer
    }

    /// returns a immutable reference to the backing buffer
    pub const fn buffer(&self) -> &T {
        &self.buffer
    }

    /// returns a mutable(!) reference to the backing buffer
    ///
    /// # Safety
    ///
    /// please do not change buffer size.
    pub unsafe fn buffer_mut(&mut self) -> &mut T {
        &mut self.buffer
    }
}

impl<const CHANNELS: usize, T: Clone> Image<&[T], CHANNELS> {
    /// Allocate a new `Image<Vec<T>>` from this imageref.
    pub fn to_owned(&self) -> Image<Vec<T>, CHANNELS> {
        // SAFETY: we have been constructed already, so must be valid
        unsafe { Image::new(self.width, self.height, self.buffer.to_vec()) }
    }
}

impl<const CHANNELS: usize, T: Clone> Image<&mut [T], CHANNELS> {
    /// Allocate a new `Image<Vec<T>>` from this mutable imageref.
    pub fn to_owned(&self) -> Image<Vec<T>, CHANNELS> {
        // SAFETY: we have been constructed already, so must be valid
        unsafe { Image::new(self.width, self.height, self.buffer.to_vec()) }
    }
}

impl<const CHANNELS: usize> Image<&[u8], CHANNELS> {
    #[inline]
    #[must_use]
    /// Copy this ref image
    pub const fn copy(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            buffer: self.buffer,
        }
    }

    /// Create a new immutable image of width x, y.
    ///
    /// # Panics
    ///
    /// if width || height == 0
    ///
    /// ```
    /// # use fimg::Image;
    /// let img = Image::make::<5, 5>();
    /// # let img: Image<_, 4> = img;
    /// ```
    pub const fn make<'a, const WIDTH: u32, const HEIGHT: u32>() -> Image<&'a [u8], CHANNELS>
    where
        [(); CHANNELS * WIDTH as usize * HEIGHT as usize]: Sized,
    {
        Image {
            width: NonZeroU32::new(WIDTH).expect("passed zero width to builder"),
            height: NonZeroU32::new(HEIGHT).expect("passed zero height to builder"),
            buffer: &[0; CHANNELS * WIDTH as usize * HEIGHT as usize],
        }
    }
}

impl<T: std::ops::Deref<Target = [u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// # Safety
    ///
    /// - UB if x, y is out of bounds
    /// - UB if buffer is too small
    #[inline]
    unsafe fn slice(&self, x: u32, y: u32) -> impl SliceIndex<[u8], Output = [u8]> {
        debug_assert!(x < self.width(), "x out of bounds");
        debug_assert!(y < self.height(), "y out of bounds");
        // SAFETY: me when uncheck math: 😧
        let index = unsafe { really_unsafe_index(x, y, self.width()) };
        // SAFETY: 🧐 is unsound? 😖
        let index = unsafe { index.unchecked_mul(CHANNELS) };
        debug_assert!(self.buffer.len() > index);
        // SAFETY: as long as the buffer isnt wrong, this is 😄
        index..unsafe { index.unchecked_add(CHANNELS) }
    }

    #[inline]
    /// Returns a iterator over every pixel
    pub fn chunked(&self) -> impl Iterator<Item = &[u8; CHANNELS]> {
        // SAFETY: 0 sized images illegal
        unsafe { assert_unchecked!(self.buffer.len() > CHANNELS) };
        // SAFETY: no half pixels!
        unsafe { assert_unchecked!(self.buffer.len() % CHANNELS == 0) };
        self.buffer.array_chunks::<CHANNELS>()
    }

    /// Return a pixel at (x, y).
    /// # Safety
    ///
    /// - UB if x, y is out of bounds
    /// - UB if buffer is too small
    #[inline]
    pub unsafe fn pixel(&self, x: u32, y: u32) -> [u8; CHANNELS] {
        // SAFETY: we have been told x, y is in bounds
        let idx = unsafe { self.slice(x, y) };
        // SAFETY: slice always returns a valid index
        let ptr = unsafe { self.buffer.get_unchecked(idx).as_ptr().cast() };
        // SAFETY: slice always returns a length of `CHANNELS`, so we `cast()` it for convenience.
        unsafe { *ptr }
    }
}

impl<T: std::ops::DerefMut<Target = [u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// Return a mutable reference to a pixel at (x, y).
    /// # Safety
    ///
    /// - UB if x, y is out of bounds
    /// - UB if buffer is too small
    #[inline]
    pub unsafe fn pixel_mut(&mut self, x: u32, y: u32) -> &mut [u8] {
        // SAFETY: we have been told x, y is in bounds.
        let idx = unsafe { self.slice(x, y) };
        // SAFETY: slice should always return a valid index
        unsafe { self.buffer.get_unchecked_mut(idx) }
    }

    #[inline]
    /// Returns a iterator over every pixel, mutably
    pub fn chunked_mut(&mut self) -> impl Iterator<Item = &mut [u8; CHANNELS]> {
        // SAFETY: 0 sized images are not allowed
        unsafe { assert_unchecked!(self.buffer.len() > CHANNELS) };
        // SAFETY: buffer cannot have half pixels
        unsafe { assert_unchecked!(self.buffer.len() % CHANNELS == 0) };
        self.buffer.array_chunks_mut::<CHANNELS>()
    }

    /// Set the pixel at x, y
    ///
    /// # Safety
    ///
    /// UB if x, y is out of bounds.
    #[inline]
    pub unsafe fn set_pixel(&mut self, x: u32, y: u32, px: [u8; CHANNELS]) {
        // SAFETY: Caller says that x, y is in bounds
        let out = unsafe { self.pixel_mut(x, y) };
        // SAFETY: px must be CHANNELS long
        unsafe { std::ptr::copy_nonoverlapping(px.as_ptr(), out.as_mut_ptr(), CHANNELS) };
    }
}

impl<const CHANNELS: usize> Image<&mut [u8], CHANNELS> {
    /// Downcast the mutable reference
    pub fn as_ref(&self) -> Image<&[u8], CHANNELS> {
        // SAFETY: we got constructed okay, parameters must be valid
        unsafe { Image::new(self.width, self.height, self.buffer) }
    }

    /// Copy this ref image
    pub fn copy(&mut self) -> Image<&mut [u8], CHANNELS> {
        #[allow(clippy::undocumented_unsafe_blocks)]
        unsafe {
            Image::new(self.width, self.height, self.buffer)
        }
    }
}

impl<const CHANNELS: usize> Image<Vec<u8>, CHANNELS> {
    /// Create a reference to this owned image
    pub fn as_ref(&self) -> Image<&[u8], CHANNELS> {
        #[allow(clippy::undocumented_unsafe_blocks)]
        unsafe {
            Image::new(self.width, self.height, &self.buffer)
        }
    }
}

impl<const CHANNELS: usize> Image<Vec<u8>, CHANNELS> {
    /// Create a mutable reference to this owned image
    pub fn as_mut(&mut self) -> Image<&mut [u8], CHANNELS> {
        #[allow(clippy::undocumented_unsafe_blocks)]
        unsafe {
            Image::new(self.width, self.height, &mut self.buffer)
        }
    }
}

impl<const CHANNELS: usize> Image<Vec<u8>, CHANNELS> {
    /// Allocates a new image
    ///
    /// # Panics
    ///
    /// if width || height == 0
    #[must_use]
    pub fn alloc(width: u32, height: u32) -> Self {
        Self {
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
            buffer: vec![0; CHANNELS * width as usize * height as usize],
        }
    }
}

/// helper macro for defining the save() method.
macro_rules! save {
    ($channels:literal == $clr:ident ($clrhuman:literal)) => {
        impl Image<Vec<u8>, $channels> {
            #[cfg(feature = "save")]
            #[doc = "Save this "]
            #[doc = $clrhuman]
            #[doc = " image."]
            pub fn save(&self, f: impl AsRef<std::path::Path>) {
                self.as_ref().save(f)
            }
        }

        impl Image<&[u8], $channels> {
            #[cfg(feature = "save")]
            #[doc = "Save this "]
            #[doc = $clrhuman]
            #[doc = " image."]
            pub fn save(&self, f: impl AsRef<std::path::Path>) {
                let p = std::fs::File::create(f).unwrap();
                let w = &mut std::io::BufWriter::new(p);
                let mut enc = png::Encoder::new(w, self.width(), self.height());
                enc.set_color(png::ColorType::$clr);
                enc.set_depth(png::BitDepth::Eight);
                enc.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
                enc.set_source_chromaticities(png::SourceChromaticities::new(
                    (0.31270, 0.32900),
                    (0.64000, 0.33000),
                    (0.30000, 0.60000),
                    (0.15000, 0.06000),
                ));
                let mut writer = enc.write_header().unwrap();
                writer.write_image_data(self.buffer).unwrap();
            }
        }
    };
}

save!(3 == Rgb("RGB"));
save!(4 == Rgba("RGBA"));
save!(2 == GrayscaleAlpha("YA"));
save!(1 == Grayscale("Y"));

#[cfg(test)]
macro_rules! img {
    [[$($v:literal),+] [$($v2:literal),+]] => {
        Image::<Vec<u8>, 1>::build(2,2).buf(vec![$($v,)+ $($v2,)+])
    }
}
#[cfg(test)]
use img;
