use super::{assert_unchecked, really_unsafe_index, Image};
use std::simd::SimdInt;
use std::simd::SimdPartialOrd;
use std::simd::{simd_swizzle, Simd};

pub trait OverlayAt<W> {
    /// Overlay with => self at coordinates x, y, without blending
    /// # Safety
    ///
    /// UB if x, y is out of bounds
    unsafe fn overlay_at(&mut self, with: &W, x: u32, y: u32) -> &mut Self;
}

pub trait Overlay<W> {
    /// Overlay with => self (does not blend)
    /// # Safety
    ///
    /// UB if a.width != b.width || a.height != b.height
    unsafe fn overlay(&mut self, with: &W) -> &mut Self;
}

#[inline]
unsafe fn blit(rgb: &mut [u8], rgba: &[u8]) {
    const LAST4: Simd<u8, 16> = Simd::from_array([
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0,
    ]);

    let mut srci = 0;
    let mut dsti = 0;
    while dsti + 16 <= rgb.len() {
        let old: Simd<u8, 16> = Simd::from_slice(unsafe { rgb.get_unchecked(dsti..dsti + 16) });
        let new: Simd<u8, 16> = Simd::from_slice(unsafe { rgba.get_unchecked(srci..srci + 16) });

        let threshold = new.simd_ge(Simd::splat(128)).to_int().cast::<u8>();
        let mut mask = simd_swizzle!(
            threshold,
            [3, 3, 3, 7, 7, 7, 11, 11, 11, 15, 15, 15, 0, 0, 0, 0]
        );
        mask &= LAST4;

        let new_rgb = simd_swizzle!(new, [0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 0, 0, 0, 0]);
        let blended = (new_rgb & mask) | (old & !mask);
        blended.copy_to_slice(unsafe { rgb.get_unchecked_mut(dsti..dsti + 16) });

        srci += 16;
        dsti += 12;
    }

    while dsti + 3 <= rgb.len() {
        if unsafe { *rgba.get_unchecked(srci + 3) } >= 128 {
            let src = unsafe { rgba.get_unchecked(srci..srci + 3) };
            let end = unsafe { rgb.get_unchecked_mut(dsti..dsti + 3) };
            unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), end.as_mut_ptr(), 3) };
        }

        srci += 4;
        dsti += 3;
    }
}

impl Overlay<Image<&[u8], 4>> for Image<&mut [u8], 4> {
    #[inline]
    unsafe fn overlay(&mut self, with: &Image<&[u8], 4>) -> &mut Self {
        debug_assert!(self.width() == with.width());
        debug_assert!(self.height() == with.height());
        for (i, other_pixels) in with.chunked().enumerate() {
            if other_pixels[3] >= 128 {
                let idx_begin = unsafe { i.unchecked_mul(4) };
                let idx_end = unsafe { idx_begin.unchecked_add(4) };
                let own_pixels = unsafe { self.buffer.get_unchecked_mut(idx_begin..idx_end) };
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        other_pixels.as_ptr(),
                        own_pixels.as_mut_ptr(),
                        4,
                    );
                };
            }
        }
        self
    }
}

impl OverlayAt<Image<&[u8], 4>> for Image<&mut [u8], 3> {
    #[inline]
    unsafe fn overlay_at(&mut self, with: &Image<&[u8], 4>, x: u32, y: u32) -> &mut Self {
        // SAFETY: caller upholds these
        unsafe { assert_unchecked!(x + with.width() <= self.width()) };
        unsafe { assert_unchecked!(y + with.height() <= self.height()) };
        for j in 0..with.height() {
            let i_x = j as usize * with.width() as usize * 4
                ..(j as usize + 1) * with.width() as usize * 4;
            let o_x = ((j as usize + y as usize) * self.width() as usize + x as usize) * 3
                ..((j as usize + y as usize) * self.width() as usize
                    + x as usize
                    + with.width() as usize)
                    * 3;
            let rgb = unsafe { self.buffer.get_unchecked_mut(o_x) };
            let rgba = unsafe { with.buffer.get_unchecked(i_x) };
            unsafe { blit(rgb, rgba) }
        }
        self
    }
}

impl OverlayAt<Image<&[u8], 3>> for Image<&mut [u8], 3> {
    #[inline]
    unsafe fn overlay_at(&mut self, with: &Image<&[u8], 3>, x: u32, y: u32) -> &mut Self {
        macro_rules! o3x3 {
            ($n:expr) => {{
                for j in 0..($n as usize) {
                    let i_x = j * ($n as usize) * 3..(j + 1) * ($n as usize) * 3;
                    let o_x = ((j + y as usize) * self.width() as usize + x as usize) * 3
                        ..((j + y as usize) * self.width() as usize + x as usize + ($n as usize))
                            * 3;
                    let a = unsafe { self.buffer.get_unchecked_mut(o_x) };
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

impl Overlay<Image<&[u8], 4>> for Image<&mut [u8], 3> {
    #[inline]
    unsafe fn overlay(&mut self, with: &Image<&[u8], 4>) -> &mut Self {
        debug_assert!(self.width() == with.width());
        debug_assert!(self.height() == with.height());
        for (i, chunk) in with
            .buffer
            .chunks_exact(with.width() as usize * 4)
            .enumerate()
        {
            let rgb = unsafe {
                self.buffer.get_unchecked_mut(
                    i * with.width() as usize * 3..(i + 1) * with.width() as usize * 3,
                )
            };
            unsafe { blit(rgb, chunk) };
        }
        self
    }
}

impl OverlayAt<Image<&[u8], 4>> for Image<&mut [u8], 4> {
    #[inline]
    unsafe fn overlay_at(&mut self, with: &Image<&[u8], 4>, x: u32, y: u32) -> &mut Self {
        for j in 0..with.height() {
            for i in 0..with.width() {
                let index = unsafe { really_unsafe_index(i, j, with.width()) };
                let their_px = unsafe { with.buffer.get_unchecked(index * 4..index * 4 + 4) };
                if unsafe { *their_px.get_unchecked(3) } >= 128 {
                    let x = unsafe { i.unchecked_add(x) };
                    let y = unsafe { j.unchecked_add(y) };
                    let index = unsafe { really_unsafe_index(x, y, self.width()) };
                    let our_px = unsafe { self.buffer.get_unchecked_mut(index * 4..index * 4 + 4) };
                    our_px.copy_from_slice(their_px);
                }
            }
        }

        self
    }
}

#[cfg(test)]
mod bench {
    extern crate test;

    use test::Bencher;

    use super::*;
    use crate::{FromRef, FromRefMut};

    #[bench]
    fn overlay_3on3at(bench: &mut Bencher) {
        let mut v = vec![0u8; 3 * 64 * 64];
        let mut a: Image<_, 3> = Image::new(
            64.try_into().unwrap(),
            64.try_into().unwrap(),
            v.as_mut_slice(),
        );
        let b = Image::<&[u8], 3>::new(
            4.try_into().unwrap(),
            4.try_into().unwrap(),
            *&include_bytes!("../test_data/3_4x4.imgbuf"),
        );
        bench.iter(|| unsafe {
            for x in 0..16 {
                for y in 0..16 {
                    a.overlay_at(&b, x * 4, y * 4);
                }
            }
        });
        assert_eq!(
            a.as_ref().buffer,
            include_bytes!("../test_results/3x3_at_out.buf")
        );
    }

    #[bench]
    fn overlay_4on3at(bench: &mut Bencher) {
        let mut a: Image<_, 3> = Image::alloc(64, 64);
        let b = Image::<&[u8], 4>::new(
            4.try_into().unwrap(),
            4.try_into().unwrap(),
            *&include_bytes!("../test_data/4_4x4.imgbuf"),
        );
        bench.iter(|| unsafe {
            for x in 0..16 {
                for y in 0..16 {
                    a.as_mut().overlay_at(&b, x * 4, y * 4);
                }
            }
        });
        assert_eq!(
            a.as_ref().buffer,
            include_bytes!("../test_results/4x3_at_out.buf")
        );
    }

    #[bench]
    fn overlay_4on4at(bench: &mut Bencher) {
        let mut a: Image<_, 4> = Image::alloc(64, 64);
        let b = Image::<&[u8], 4>::new(
            4.try_into().unwrap(),
            4.try_into().unwrap(),
            *&include_bytes!("../test_data/4_4x4.imgbuf"),
        );
        bench.iter(|| unsafe {
            for x in 0..16 {
                for y in 0..16 {
                    a.as_mut().overlay_at(&b, x * 4, y * 4);
                }
            }
        });
        assert_eq!(
            a.as_ref().buffer,
            include_bytes!("../test_results/4x4_at_out.buf")
        );
    }
}
