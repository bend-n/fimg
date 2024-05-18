#![allow(clippy::undocumented_unsafe_blocks)]
use core::intrinsics::simd::simd_cast;
use std::{
    arch::x86_64::*,
    intrinsics::transmute_unchecked,
    simd::{prelude::*, LaneCount, MaskElement, SimdElement, SupportedLaneCount},
};

#[test]
fn b64() {
    fn t(i: &'static str, o: &'static str) {
        let mut x = Vec::with_capacity(size(i.as_bytes()));
        unsafe { portable(i.as_bytes(), x.as_mut_ptr()) };
        unsafe { x.set_len(size(i.as_bytes())) };
        assert_eq!(x, o.as_bytes());
    }

    t("Hello World!", "SGVsbG8gV29ybGQh");
    t("Hello World", "SGVsbG8gV29ybGQ=");
}

pub fn encode(i: &[u8]) -> String {
    let mut x = Vec::with_capacity(size(i));
    unsafe { portable(i, x.as_mut_ptr()) };
    unsafe { x.set_len(size(i)) };
    unsafe { String::from_utf8_unchecked(x) }
}

trait Cast<T, const N: usize> {
    fn cas<U: SimdT>(self) -> U;
}
trait SimdT {}
impl<T: SimdElement, const N: usize> SimdT for Simd<T, N> where LaneCount<N>: SupportedLaneCount {}
impl<T: MaskElement, const N: usize> SimdT for Mask<T, N> where LaneCount<N>: SupportedLaneCount {}
impl<T: SimdElement, const N: usize> Cast<T, N> for Simd<T, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn cas<U>(self) -> U {
        assert!(std::mem::size_of::<U>() == std::mem::size_of::<Self>());
        unsafe { transmute_unchecked(self) }
    }
}

impl<T: MaskElement, const N: usize> Cast<T, N> for Mask<T, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn cas<U>(self) -> U {
        assert!(std::mem::size_of::<U>() == std::mem::size_of::<Self>());
        unsafe { transmute_unchecked(self) }
    }
}

#[allow(non_camel_case_types)]
type c = u8x32;
unsafe fn portable(mut input: &[u8], mut output: *mut u8) {
    while input.len() >= 32 {
        #[allow(unsafe_op_in_unsafe_fn)]
        let indices = if cfg!(all(target_feature = "avx2", not(miri))) {
            let lo = _mm_loadu_si128(input.as_ptr() as *const __m128i);
            let hi = _mm_loadu_si128(input.as_ptr().add(12) as *const __m128i);
            let i = _mm256_shuffle_epi8(
                _mm256_set_m128i(hi, lo),
                _mm256_set_epi8(
                    10, 11, 9, 10, 7, 8, 6, 7, 4, 5, 3, 4, 1, 2, 0, 1, //
                    10, 11, 9, 10, 7, 8, 6, 7, 4, 5, 3, 4, 1, 2, 0, 1, //
                ),
            );
            let t0 = _mm256_and_si256(i, _mm256_set1_epi32(0x0fc0fc00));
            let t1 = _mm256_mulhi_epu16(t0, _mm256_set1_epi32(0x04000040));
            let t2 = _mm256_and_si256(i, _mm256_set1_epi32(0x003f03f0));
            let t3 = _mm256_mullo_epi16(t2, _mm256_set1_epi32(0x01000010));

            c::from(_mm256_or_si256(t1, t3))
        } else {
            let v = c::from_slice(input);
            let i = simd_swizzle!(
                v,
                [
                    1, 0, 2, 1, 4, 3, 5, 4, 7, 6, 8, 7, 10, 9, 11, 10, 13, 12, //
                    14, 13, 16, 15, 17, 16, 19, 18, 20, 19, 22, 21, 23, 22
                ]
            );

            // https://github.com/WojciechMula/base64simd
            let t0 = i & u32x8::splat(0x0fc0fc00).cas::<c>();
            let t1 = Cast::cas::<c>(mulhi(t0.cas(), u32x8::splat(0x04000040).cas()));
            let t2 = i & u32x8::splat(0x003f03f0).cas::<c>();
            let t3 = mullo(t2.cas(), u32x8::splat(0x01000010).cas()).cas::<c>();
            t1 | t3
        };
        lookup(indices).copy_to_slice(unsafe { std::slice::from_raw_parts_mut(output, 32) });
        output = unsafe { output.add(32) };

        input = &input[24..];
    }
    unsafe { encode_simple(input, output) };
}

fn mulhi(x: u16x16, y: u16x16) -> u16x16 {
    unsafe {
        simd_cast::<_, u16x16>(
            simd_cast::<_, u32x16>(x) * simd_cast::<_, u32x16>(y) >> u32x16::splat(16),
        )
    }
}

fn mullo(x: u16x16, y: u16x16) -> u16x16 {
    x * y
}

fn lookup(x: c) -> c {
    let result = x.saturating_sub(c::splat(51));
    let less = cmpgt(c::splat(26), x);
    let result = result | (less & c::splat(13));

    #[rustfmt::skip]
    const LUT: i8x32 = i8x32::from_array([
        b'a' as i8 - 26, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52,
        b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'+' as i8 - 62,
        b'/' as i8 - 63, b'A' as i8, 0, 0,

        b'a' as i8 - 26, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52,
        b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'0' as i8 - 52, b'+' as i8 - 62,
        b'/' as i8 - 63, b'A' as i8, 0, 0
    ]);
    let result = if cfg!(all(target_feature = "avx2", not(miri))) {
        unsafe { i8x32::from(_mm256_shuffle_epi8(LUT.into(), result.into())) }
    } else {
        (LUT.cas::<c>().swizzle_dyn(result)).cas::<i8x32>()
    };

    Cast::cas(result + x.cas::<i8x32>())
}

pub fn cmpgt(x: c, y: c) -> c {
    x.cas::<i8x32>().simd_gt(y.cas::<i8x32>()).cas()
}

trait P {
    unsafe fn p<const N: usize>(&mut self, data: [u8; N]);
}

impl P for *mut u8 {
    unsafe fn p<const N: usize>(&mut self, data: [u8; N]) {
        unsafe { self.copy_from(data.as_ptr(), N) };
        *self = unsafe { self.add(N) };
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
/// # Safety
///  ptr valid for [`size`]`(input)` writes.
pub unsafe fn encode_simple(mut input: &[u8], mut output: *mut u8) {
    const Α: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    while let [a, b, c, rest @ ..] = input {
        let α = ((*a as usize) << 16) | ((*b as usize) << 8) | *c as usize;
        output.p([
            Α[α >> 18],
            Α[(α >> 12) & 0x3F],
            Α[(α >> 6) & 0x3F],
            Α[α & 0x3F],
        ]);
        input = rest;
    }
    if !input.is_empty() {
        let mut α = (input[0] as usize) << 16;
        if input.len() > 1 {
            α |= (input[1] as usize) << 8;
        }
        output.p([Α[α >> 18], Α[α >> 12 & 0x3F]]);
        if input.len() > 1 {
            output.p([Α[α >> 6 & 0x3f]]);
        } else {
            output.p([b'=']);
        }
        output.p([b'=']);
    }
}

pub const fn size(of: &[u8]) -> usize {
    let use_pad = of.len() % 3 != 0;
    if use_pad {
        4 * (of.len() / 3 + 1)
    } else {
        4 * (of.len() / 3)
    }
}
