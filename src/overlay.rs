//! Handles image overlay
use crate::cloner::ImageCloner;

use super::{assert_unchecked, Image};
use std::ops::{Deref, DerefMut};
use std::simd::{simd_swizzle, Simd, SimdInt, SimdPartialOrd};

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
    unsafe fn overlay_at(&self, with: &Image<&[u8], W>, x: u32, y: u32) -> Image<Vec<u8>, C>;
}

/// Trait for layering images ontop of each other.
/// Think `magick a b -layers flatten a`
pub trait Overlay<W> {
    /// Overlay with => self (does not blend)
    /// # Safety
    ///
    /// UB if a.width != b.width || a.height != b.height
    unsafe fn overlay(&mut self, with: &W) -> &mut Self;
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
unsafe fn blit(rgb: &mut [u8], rgba: &[u8]) {
    let mut srci = 0;
    let mut dsti = 0;
    while dsti + 16 <= rgb.len() {
        // SAFETY: i think it ok
        let old: Simd<u8, 16> = Simd::from_slice(unsafe { rgb.get_unchecked(dsti..dsti + 16) });
        // SAFETY: definetly ok
        let new: Simd<u8, 16> = Simd::from_slice(unsafe { rgba.get_unchecked(srci..srci + 16) });

        let threshold = new.simd_ge(Simd::splat(128)).to_int().cast::<u8>();
        let mut mask = simd_swizzle!(
            threshold,
            [3, 3, 3, 7, 7, 7, 11, 11, 11, 15, 15, 15, 0, 0, 0, 0]
        );
        mask &= Simd::from_array([
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0,
        ]);

        let new_rgb = simd_swizzle!(new, [0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 0, 0, 0, 0]);
        let blended = (new_rgb & mask) | (old & !mask);
        // SAFETY: 4 * 4 == 16, so in bounds
        blended.copy_to_slice(unsafe { rgb.get_unchecked_mut(dsti..dsti + 16) });

        srci += 16;
        dsti += 12;
    }

    while dsti + 3 <= rgb.len() {
        // SAFETY: caller gurantees slice is big enough
        if unsafe { *rgba.get_unchecked(srci + 3) } >= 128 {
            // SAFETY: slice is big enough!
            let src = unsafe { rgba.get_unchecked(srci..=srci + 2) };
            // SAFETY: i hear it bound
            let end = unsafe { rgb.get_unchecked_mut(dsti..=dsti + 2) };
            end.copy_from_slice(src);
        }

        srci += 4;
        dsti += 3;
    }
}

impl<T: DerefMut<Target = [u8]>, U: Deref<Target = [u8]>> Overlay<Image<U, 4>> for Image<T, 4> {
    #[inline]
    unsafe fn overlay(&mut self, with: &Image<U, 4>) -> &mut Self {
        debug_assert!(self.width() == with.width());
        debug_assert!(self.height() == with.height());
        for (i, other_pixels) in with.chunked().enumerate() {
            if other_pixels[3] >= 128 {
                // SAFETY: outside are bounds of index from slice
                let own_pixels = unsafe { self.buffer.get_unchecked_mut(i * 4..i * 4 + 4) };
                own_pixels.copy_from_slice(other_pixels);
            }
        }
        self
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

impl<T: DerefMut<Target = [u8]>, U: Deref<Target = [u8]>> OverlayAt<Image<U, 4>> for Image<T, 3> {
    #[inline]
    unsafe fn overlay_at(&mut self, with: &Image<U, 4>, x: u32, y: u32) -> &mut Self {
        // SAFETY: caller upholds this
        unsafe { assert_unchecked!(x + with.width() <= self.width()) };
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
            let rgb = unsafe { self.buffer.get_unchecked_mut(o_x) };
            // SAFETY: bounds are outside index
            let rgba = unsafe { with.buffer.get_unchecked(i_x) };
            // SAFETY: arguments are 🟢
            unsafe { blit(rgb, rgba) }
        }
        self
    }
}

impl ClonerOverlayAt<4, 3> for ImageCloner<'_, 3> {
    #[inline]
    #[must_use = "function does not modify the original image"]
    unsafe fn overlay_at(&self, with: &Image<&[u8], 4>, x: u32, y: u32) -> Image<Vec<u8>, 3> {
        let mut new = self.dup();
        // SAFETY: same
        unsafe { new.as_mut().overlay_at(with, x, y) };
        new
    }
}

impl<T: DerefMut<Target = [u8]>, U: Deref<Target = [u8]>> OverlayAt<Image<U, 3>> for Image<T, 3> {
    /// Overlay a RGB image(with) => self at coordinates x, y.
    /// As this is a `RGBxRGB` operation, blending is unnecessary,
    /// and this is simply a copy.
    ///
    /// # Safety
    ///
    /// UB if x, y is out of bounds
    #[inline]
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
                    debug_assert!(o_x.end <= self.buffer().len());
                    debug_assert!(i_x.end <= with.buffer().len());
                    // SAFETY: bounds are ✅
                    let a = unsafe { self.buffer.get_unchecked_mut(o_x) };
                    // SAFETY: we are in ⬜!
                    let b = unsafe { with.buffer.get_unchecked(i_x) };
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

impl<T: DerefMut<Target = [u8]>, U: Deref<Target = [u8]>> Overlay<Image<U, 4>> for Image<T, 3> {
    #[inline]
    unsafe fn overlay(&mut self, with: &Image<U, 4>) -> &mut Self {
        debug_assert!(self.width() == with.width());
        debug_assert!(self.height() == with.height());
        for (i, chunk) in with
            .buffer
            .chunks_exact(with.width() as usize * 4)
            .enumerate()
        {
            // SAFETY: all the bounds are good
            let rgb = unsafe {
                self.buffer.get_unchecked_mut(
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

impl<T: DerefMut<Target = [u8]>, U: Deref<Target = [u8]>> OverlayAt<Image<U, 4>> for Image<T, 4> {
    #[inline]
    /// Overlay with => self at coordinates x, y, without blending
    ///
    /// # Safety
    /// - UB if x, y is out of bounds
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
    /// Overlay with => self at coordinates x, y, without blending, returning a new Image
    ///
    /// # Safety
    /// - UB if x, y is out of bounds
    /// - UB if x + with.width() > [`u32::MAX`]
    /// - UB if y + with.height() > [`u32::MAX`]
    unsafe fn overlay_at(&self, with: &Image<&[u8], 4>, x: u32, y: u32) -> Image<Vec<u8>, 4> {
        let mut out = self.dup();
        // SAFETY: same
        unsafe { out.as_mut().overlay_at(with, x, y) };
        out
    }
}
