//! define From's for images.
//! these conversions are defined by [`PFrom`].
use crate::{Image, Pack, pixels::convert::PFrom};
use array_chunks::*;
use core::intrinsics::{fmul_algebraic, fsub_algebraic, transmute_unchecked as transmute};
use std::{
    mem::MaybeUninit as MU,
    simd::{SimdElement, StdFloat, prelude::*},
};

fn map<const A: usize, const B: usize>(image: Image<&[u8], A>) -> Image<Box<[u8]>, B>
where
    [u8; B]: PFrom<A>,
{
    // SAFETY: size unchanged, just change pixels
    unsafe {
        image.mapped(|buf| {
            buf.array_chunks::<A>()
                .copied()
                .flat_map(<[u8; B] as PFrom<A>>::pfrom)
                .collect()
        })
    }
}

macro_rules! convert {
    ($a:literal => $b:literal) => {
        impl From<Image<&[u8], $b>> for Image<Box<[u8]>, $a> {
            fn from(value: Image<&[u8], $b>) -> Self {
                map(value)
            }
        }
    };
}

macro_rules! cv {
    [$($n:literal),+] => {
        $(convert!($n => 1);
        convert!($n => 2);
        convert!($n => 3);
        convert!($n => 4);)+
    };
}

cv![1, 2, 3, 4];

macro_rules! boxconv {
    ($a:literal => $b: literal) => {
        impl From<Image<Box<[u8]>, $b>> for Image<Box<[u8]>, $a> {
            fn from(value: Image<Box<[u8]>, $b>) -> Self {
                value.as_ref().into()
            }
        }
    };
}

boxconv!(1 => 2);
boxconv!(1 => 3);
boxconv!(1 => 4);

boxconv!(2 => 1);
boxconv!(2 => 3);
boxconv!(2 => 4);

boxconv!(3 => 1);
boxconv!(3 => 2);
boxconv!(3 => 4);

boxconv!(4 => 1);
boxconv!(4 => 2);
boxconv!(4 => 3);

impl<const N: usize> From<Image<&[u8], N>> for Image<Box<[u32]>, 1>
where
    [u8; N]: Pack,
{
    /// Pack into ARGB.
    fn from(value: Image<&[u8], N>) -> Self {
        let buf = value.chunked().map(Pack::pack).collect();
        // SAFETY: ctor
        unsafe { Self::new(value.width, value.height, buf) }
    }
}

pub fn unpack_all<const N: usize>(buffer: &[u32]) -> impl Iterator<Item = u8> + '_
where
    [u8; N]: Pack,
{
    buffer.iter().copied().flat_map(<[u8; N]>::unpack)
}

impl<const N: usize> From<Image<&[u32], 1>> for Image<Box<[u8]>, N>
where
    [u8; N]: Pack,
{
    fn from(value: Image<&[u32], 1>) -> Self {
        let buf = unpack_all(value.buffer).collect();
        // SAFETY: ctor
        unsafe { Self::new(value.width, value.height, buf) }
    }
}

fn u8_to_f32(x: u8) -> f32 {
    let magic = 2.0f32.powf(23.);
    // x = 2^23 + x
    let x = f32::from_bits((x as u32) ^ magic.to_bits());
    fmul_algebraic(fsub_algebraic(x, magic), 1.0 / 255.0)
}

fn u8s_to_f32s(x: u8x8) -> f32x8 {
    let x = x.cast::<u32>();
    let magic = (1 << 23) as f32;
    // SAFETY: its a simd, i can do what i want with it
    let x = unsafe { transmute::<_, f32x8>(x ^ Simd::splat(magic.to_bits())) };
    x.mul_add(Simd::splat(1.0 / 255.0), Simd::splat(-magic / 255.0))
}

// notice: this f32 better be in range 0.0-1.0
fn f32_to_u8(x: f32) -> u8 {
    let magic = (1 << 23) as f32;
    (x.mul_add(255.0, magic).to_bits() ^ magic.to_bits()) as u8
}

fn f32s_to_u8s(x: f32x8) -> u8x8 {
    let magic = (1 << 23) as f32;
    (x.mul_add(Simd::splat(255.0), Simd::splat(magic)).cast() ^ Simd::splat(magic.to_bits())).cast()
}

fn mapping<T, U>(
    x: &[T],
    mut f: impl FnMut(Simd<T, 8>) -> Simd<U, 8>,
    mut single: impl FnMut(T) -> U,
) -> Vec<U>
where
    T: SimdElement,
    U: SimdElement,
    [(); (size_of::<Simd<T, 8>>() == size_of::<[T; 8]>()) as usize - 1]:,
{
    let mut out = Vec::with_capacity(x.len());
    let to = out.spare_capacity_mut();
    let (to, to_rest) = to.as_chunks_mut::<8>();
    let (from, from_rest) = x.as_chunks::<8>();
    for (&line, into) in from.iter().zip(to) {
        // SAFETY: safe transmute (see condition)
        unsafe { *into = transmute::<_, [MU<U>; 8]>(f(Simd::from_array(line))) };
    }
    for (i, &from) in from_rest.iter().enumerate() {
        // SAFETY: compiler doesnt like it when i zip this
        unsafe { to_rest.get_unchecked_mut(i) }.write(single(from));
    }
    // SAFETY: initialized.
    unsafe { out.set_len(x.len()) };
    out
}

impl<const N: usize> From<Image<&[u8], N>> for Image<Box<[f32]>, N> {
    /// Reduce to 0.0-1.0 from 0-255.
    fn from(value: Image<&[u8], N>) -> Self {
        // SAFETY: length unchanged
        unsafe { value.mapped(|x| mapping(x, u8s_to_f32s, u8_to_f32).into_boxed_slice()) }
    }
}

impl<const N: usize> From<Image<&[f32], N>> for Image<Box<[u8]>, N> {
    /// Expand to 0-255 from 0.0-1.0
    fn from(value: Image<&[f32], N>) -> Self {
        // SAFETY: length unchanged
        unsafe { value.mapped(|x| mapping(x, f32s_to_u8s, f32_to_u8).into_boxed_slice()) }
    }
}

#[test]
fn roundtrip() {
    let original = Image::<_, 3>::open("tdata/small_cat.png");
    assert!(
        Image::<Box<[u8]>, 3>::from(Image::<Box<[f32]>, 3>::from(original.as_ref()).as_ref(),)
            // .show()
            .bytes()
            == original.bytes()
    );
}

impl<const N: usize, T: AsRef<[u8]>> Image<T, N> {
    /// just an `into` wrapper
    pub fn to_f32(&self) -> Image<Box<[f32]>, N> {
        self.as_ref().into()
    }
}

impl<const N: usize, T: AsRef<[f32]>> Image<T, N> {
    /// just an `into` wrapper
    pub fn to_u8(&self) -> Image<Box<[u8]>, N> {
        self.as_ref().into()
    }
}
