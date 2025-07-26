//! Handles image overlay
// TODO Y/YA
use crate::{cloner::ImageCloner, uninit};

use super::{Image, assert_unchecked};
use crate::pixels::Blend;
use std::{mem::transmute, simd::prelude::*};

/// Trait for layering a image ontop of another, with a offset to the second image.
pub trait OverlayAt<W> {
    /// Overlay with => self at coordinates x, y, without blending
    /// # Safety
    ///
    /// UB if x, y is out of bounds
    unsafe fn overlay_at(&mut self, with: &W, x: u32, y: u32) -> &mut Self;
}

/// Sealant module
mod sealed {
    /// Seals the cloner traits
    pub trait Sealed {}
}
use sealed::Sealed;
impl<const N: usize> Sealed for ImageCloner<'_, N> {}

/// [`OverlayAt`] but owned
pub trait ClonerOverlayAt<const W: usize, const C: usize>: Sealed {
    /// Overlay with => self at coordinates x, y, without blending, and returning a new image.
    /// # Safety
    ///
    /// UB if x, y is out of bounds
    #[must_use = "function does not modify the original image"]
    unsafe fn overlay_at(&self, with: &Image<&[u8], W>, x: u32, y: u32) -> Image<Vec<u8>, C>;
}

/// Trait for layering images ontop of each other.
/// Think `magick a b -layers flatten a`
pub trait Overlay<W> {
    /// Overlay with => self (does not blend)
    ///
    /// # Safety
    ///
    /// UB if a.width != b.width || a.height != b.height
    unsafe fn overlay(&mut self, with: &W) -> &mut Self;
}

/// This blends the images together, like [`imageops::overlay`](https://docs.rs/image/latest/image/imageops/fn.overlay.html).
pub trait BlendingOverlay<W> {
    /// Overlay with => self, blending. You probably do not need this, unless your images make much usage of alpha.
    /// If you only have 2 alpha states, `0` | `255` (transparent | opaque), please use [`Overlay`], as it is much faster.
    /// # Safety
    ///
    /// UB if a.width != b.width || a.height != b.height
    unsafe fn overlay_blended(&mut self, with: &W) -> &mut Self;
}
/// Blending overlay at.
pub trait BlendingOverlayAt<W> {
    /// See [BlendingOverlay::overlay_blended].
    /// # Safety
    ///
    /// UB if x, y is out of bounds
    unsafe fn overlay_blended_at(&mut self, with: &W, x: u32, y: u32) -> &mut Self;
}

/// [`Overlay`] but owned
pub trait ClonerOverlay<const W: usize, const C: usize>: Sealed {
    /// Overlay with => self (does not blend)
    /// # Safety
    ///
    /// UB if a.width != b.width || a.height != b.height
    unsafe fn overlay(&self, with: &Image<&[u8], W>) -> Image<Vec<u8>, C>;
}

#[inline]
/// SIMD accelerated rgba => rgb overlay.
///
/// See [blit](https://en.wikipedia.org/wiki/Bit_blit)
///
/// # Safety
/// - UB if rgb.len() % 3 != 0
/// - UB if rgba.len() % 4 != 0
unsafe fn blit(mut rgb: &mut [u8], mut rgba: &[u8]) {
    while rgb.len() >= 16 {
        let dst = rgb.first_chunk_mut::<16>().unwrap();
        let src = rgba.first_chunk::<16>().unwrap();
        let old = Simd::from_slice(dst);
        let new: u8x16 = Simd::from_slice(src);

        let threshold = new.simd_ge(Simd::splat(128)).to_int().cast::<u8>();
        let mut mask = simd_swizzle!(
            threshold,
            // [r, g, b, a (3)] [r, g, b, a(7)]
            [3, 3, 3, 7, 7, 7, 11, 11, 11, 15, 15, 15, 0, 0, 0, 0]
        );
        mask &= Simd::from_array([
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0,
        ]);
        // [r(0), g, b] <skip a> [r(4), g, b]
        let new_rgb = simd_swizzle!(new, [0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 0, 0, 0, 0]);
        let blended = (new_rgb & mask) | (old & !mask);
        blended.copy_to_slice(dst);
        rgb = &mut rgb[12..];
        rgba = &rgba[16..];
    }
    while rgb.len() >= 3 {
        // SAFETY: guaranteed
        if unsafe { *rgba.get_unchecked(3) } >= 128 {
            rgb[..3].copy_from_slice(&rgba[..3]);
        }
        rgba = &rgba[4..];
        rgb = &mut rgb[3..];
    }
}

impl<T: AsMut<[u8]> + AsRef<[u8]>, U: AsRef<[u8]>> Overlay<Image<U, 4>> for Image<T, 4> {
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    unsafe fn overlay(&mut self, with: &Image<U, 4>) -> &mut Self {
        debug_assert!(self.width() == with.width());
        debug_assert!(self.height() == with.height());
        for (i, other_pixels) in with.chunked().enumerate() {
            if other_pixels[3] >= 128 {
                // SAFETY: outside are bounds of index from slice
                let own_pixels =
                    unsafe { self.buffer.as_mut().get_unchecked_mut(i * 4..i * 4 + 4) };
                own_pixels.copy_from_slice(other_pixels);
            }
        }
        self
    }
}

impl<const A: usize, const B: usize, T: AsMut<[u8]> + AsRef<[u8]>, U: AsRef<[u8]>>
    BlendingOverlay<Image<U, B>> for Image<T, A>
where
    [u8; A]: Blend<B>,
{
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    unsafe fn overlay_blended(&mut self, with: &Image<U, B>) -> &mut Self {
        debug_assert!(self.width() == with.width());
        debug_assert!(self.height() == with.height());
        for (other_pixels, own_pixels) in with.chunked().zip(self.chunked_mut()) {
            own_pixels.blend(*other_pixels);
        }
        self
    }
}

impl<const A: usize, const B: usize, T: AsMut<[u8]> + AsRef<[u8]>, U: AsRef<[u8]>>
    BlendingOverlayAt<Image<U, B>> for Image<T, A>
where
    [u8; A]: Blend<B>,
{
    #[inline]
    unsafe fn overlay_blended_at(&mut self, with: &Image<U, B>, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                // SAFETY: i, j is in bounds.
                let their_px = unsafe { &with.pixel(i, j) };
                let our_px = unsafe { self.pixel_mut(i + x, j + y) };
                our_px.blend(*their_px);
            }
        }
        self
    }
}

impl<T: AsMut<[u8]> + AsRef<[u8]>> Image<T, 3> {
    #[doc(hidden)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub unsafe fn blend_alpha_and_color_at(
        &mut self,
        with: &Image<&[u8], 1>,
        color: [u8; 3],
        x: u32,
        y: u32,
    ) {
        for j in 0..with.height() {
            for i in 0..with.width() {
                let &[their_alpha] = unsafe { &with.pixel(i, j) };
                let our_pixel = unsafe { self.pixel_mut(i + x, j + y) };
                crate::pixels::blending::blend_alpha_and_color(their_alpha, color, our_pixel);
            }
        }
    }
}

impl ClonerOverlay<4, 4> for ImageCloner<'_, 4> {
    #[inline]
    #[must_use = "function does not modify the original image"]
    unsafe fn overlay(&self, with: &Image<&[u8], 4>) -> Image<Vec<u8>, 4> {
        let mut out = self.dup();
        // SAFETY: same
        unsafe { out.as_mut().overlay(with) };
        out
    }
}

impl<T: AsMut<[u8]> + AsRef<[u8]>, U: AsRef<[u8]>> OverlayAt<Image<U, 4>> for Image<T, 3> {
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    unsafe fn overlay_at(&mut self, with: &Image<U, 4>, x: u32, y: u32) -> &mut Self {
        // SAFETY: caller upholds this
        unsafe { assert_unchecked(x + with.width() <= self.width()) };
        debug_assert!(y + with.height() <= self.height());
        for j in 0..with.height() {
            let i_x = j as usize * with.width() as usize * 4
                ..(j as usize + 1) * with.width() as usize * 4;
            let o_x = ((j as usize + y as usize) * self.width() as usize + x as usize) * 3
                ..((j as usize + y as usize) * self.width() as usize
                    + x as usize
                    + with.width() as usize)
                    * 3;
            // SAFETY: index is in bounds
            let rgb = unsafe { self.buffer.as_mut().get_unchecked_mut(o_x) };
            // SAFETY: bounds are outside index
            let rgba = unsafe { with.buffer.as_ref().get_unchecked(i_x) };
            // SAFETY: arguments are 🟢
            unsafe { blit(rgb, rgba) }
        }
        self
    }
}

impl<U: AsRef<[u8]>> OverlayAt<Image<U, 4>> for uninit::Image<u8, 3> {
    #[cfg_attr(debug_assertions, track_caller)]
    unsafe fn overlay_at(&mut self, with: &Image<U, 4>, x: u32, y: u32) -> &mut Self {
        // SAFETY: caller upholds this
        unsafe { assert_unchecked(x + with.width() <= self.width()) };
        debug_assert!(y + with.height() <= self.height());
        for j in 0..with.height() {
            let i_x = j as usize * with.width() as usize * 4
                ..(j as usize + 1) * with.width() as usize * 4;
            let o_x = ((j as usize + y as usize) * self.width() as usize + x as usize) * 3
                ..((j as usize + y as usize) * self.width() as usize
                    + x as usize
                    + with.width() as usize)
                    * 3;
            // SAFETY: index is in bounds
            let rgb = unsafe { transmute(self.buf().get_unchecked_mut(o_x)) };
            // SAFETY: bounds are outside index
            let rgba = unsafe { with.buffer.as_ref().get_unchecked(i_x) };
            // SAFETY: arguments are 🟢
            unsafe { blit(rgb, rgba) }
        }
        self
    }
}

impl ClonerOverlayAt<4, 3> for ImageCloner<'_, 3> {
    #[inline]
    unsafe fn overlay_at(&self, with: &Image<&[u8], 4>, x: u32, y: u32) -> Image<Vec<u8>, 3> {
        let mut new = self.dup();
        // SAFETY: same
        unsafe { new.as_mut().overlay_at(with, x, y) };
        new
    }
}

impl<U: AsRef<[u8]>> OverlayAt<Image<U, 3>> for uninit::Image<u8, 3> {
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    unsafe fn overlay_at(&mut self, with: &Image<U, 3>, x: u32, y: u32) -> &mut Self {
        for j in 0..(with.width() as usize) {
            let i_x = j * (with.width() as usize) * 3..(j + 1) * (with.width() as usize) * 3;
            let o_x = ((j + y as usize) * self.width() as usize + x as usize) * 3
                ..((j + y as usize) * self.width() as usize + x as usize + (with.width() as usize))
                    * 3;
            // <= because ".." range
            // debug_assert!(o_x.end <= self.buffer().as_ref().len());
            debug_assert!(i_x.end <= with.buffer().as_ref().len());
            // SAFETY: we are in ⬜!
            let b = unsafe { with.buffer.as_ref().get_unchecked(i_x) };
            // SAFETY: should work
            unsafe { self.write(b, o_x) };
        }
        self
    }
}

impl<T: AsMut<[u8]> + AsRef<[u8]>, U: AsRef<[u8]>> OverlayAt<Image<U, 3>> for Image<T, 3> {
    /// Overlay a RGB image(with) => self at coordinates x, y.
    /// As this is a `RGBxRGB` operation, blending is unnecessary,
    /// and this is simply a copy.
    ///
    /// # Safety
    ///
    /// UB if x, y is out of bounds
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    unsafe fn overlay_at(&mut self, with: &Image<U, 3>, x: u32, y: u32) -> &mut Self {
        /// helper macro for defining rgb=>rgb overlays. allows unrolling
        macro_rules! o3x3 {
            ($n:expr) => {{
                for j in 0..($n as usize) {
                    let i_x = j * ($n as usize) * 3..(j + 1) * ($n as usize) * 3;
                    let o_x = ((j + y as usize) * self.width() as usize + x as usize) * 3
                        ..((j + y as usize) * self.width() as usize + x as usize + ($n as usize))
                            * 3;
                    // <= because ".." range
                    debug_assert!(o_x.end <= self.buffer().as_ref().len());
                    debug_assert!(i_x.end <= with.buffer().as_ref().len());
                    // SAFETY: bounds are ✅
                    let a = unsafe { self.buffer.as_mut().get_unchecked_mut(o_x) };
                    // SAFETY: we are in ⬜!
                    let b = unsafe { with.buffer.as_ref().get_unchecked(i_x) };
                    a.copy_from_slice(b);
                }
            }};
        }
        // let it unroll
        match with.width() {
            8 => o3x3!(8),
            16 => o3x3!(16), // this branch makes 8x8 0.16 times slower; but 16x16 0.2 times faster.
            _ => o3x3!(with.width()),
        }
        self
    }
}

impl ClonerOverlayAt<3, 3> for ImageCloner<'_, 3> {
    /// Overlay a RGB image(with) => self at coordinates x, y.
    /// As this is a `RGBxRGB` operation, blending is unnecessary,
    /// and this is simply a copy.
    ///
    /// # Safety
    ///
    /// UB if x, y is out of bounds
    #[inline]
    unsafe fn overlay_at(&self, with: &Image<&[u8], 3>, x: u32, y: u32) -> Image<Vec<u8>, 3> {
        let mut out = self.dup();
        // SAFETY: same
        unsafe { out.as_mut().overlay_at(with, x, y) };
        out
    }
}

impl<T: AsMut<[u8]> + AsRef<[u8]>, U: AsRef<[u8]>> Overlay<Image<U, 4>> for Image<T, 3> {
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    unsafe fn overlay(&mut self, with: &Image<U, 4>) -> &mut Self {
        debug_assert!(self.width() == with.width());
        debug_assert!(self.height() == with.height());
        for (i, chunk) in with
            .buffer
            .as_ref()
            .chunks_exact(with.width() as usize * 4)
            .enumerate()
        {
            // SAFETY: all the bounds are good
            let rgb = unsafe {
                self.buffer.as_mut().get_unchecked_mut(
                    i * with.width() as usize * 3..(i + 1) * with.width() as usize * 3,
                )
            };
            // SAFETY: we have the rgb and rgba arguments right
            unsafe { blit(rgb, chunk) };
        }
        self
    }
}

impl ClonerOverlay<4, 3> for ImageCloner<'_, 3> {
    #[inline]
    #[must_use = "function does not modify the original image"]
    unsafe fn overlay(&self, with: &Image<&[u8], 4>) -> Image<Vec<u8>, 3> {
        let mut out = self.dup();
        // SAFETY: same
        unsafe { out.as_mut().overlay(with) };
        out
    }
}

impl<T: AsMut<[u8]> + AsRef<[u8]>, U: AsRef<[u8]>> OverlayAt<Image<U, 4>> for Image<T, 4> {
    #[inline]
    unsafe fn overlay_at(&mut self, with: &Image<U, 4>, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                // SAFETY: i, j is in bounds.
                let their_px = unsafe { &with.pixel(i, j) };
                if their_px[3] >= 128 {
                    // SAFETY: if everything else goes well, this is fine
                    let our_px = unsafe { self.pixel_mut(i + x, j + y) };
                    our_px.copy_from_slice(their_px);
                }
            }
        }

        self
    }
}

impl ClonerOverlayAt<4, 4> for ImageCloner<'_, 4> {
    #[inline]
    #[must_use = "function does not modify the original image"]
    unsafe fn overlay_at(&self, with: &Image<&[u8], 4>, x: u32, y: u32) -> Image<Vec<u8>, 4> {
        let mut out = self.dup();
        // SAFETY: same
        unsafe { out.as_mut().overlay_at(with, x, y) };
        out
    }
}
