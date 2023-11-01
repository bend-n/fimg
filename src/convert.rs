//! define From's for images.
//! these conversions are defined by [`PFrom`].
use crate::{pixels::convert::PFrom, Image};

fn map<const A: usize, const B: usize>(image: Image<&[u8], A>) -> Image<Box<[u8]>, B>
where
    [u8; B]: PFrom<A>,
{
    let buffer = image
        .chunked()
        .copied()
        .flat_map(<[u8; B] as PFrom<A>>::pfrom)
        .collect::<Vec<_>>()
        .into();
    // SAFETY: ctor
    unsafe { Image::new(image.width, image.height, buffer) }
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

#[inline]
const fn pack([r, g, b, a]: [u8; 4]) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[inline]
const fn unpack(n: u32) -> [u8; 4] {
    [
        ((n >> 16) & 0xFF) as u8,
        ((n >> 8) & 0xFF) as u8,
        (n & 0xFF) as u8,
        ((n >> 24) & 0xFF) as u8,
    ]
}

impl<const N: usize> From<Image<&[u8], N>> for Image<Box<[u32]>, 1>
where
    [u8; 4]: PFrom<N>,
{
    /// Pack into ARGB.
    fn from(value: Image<&[u8], N>) -> Self {
        let buf = value
            .chunked()
            .copied()
            .map(PFrom::pfrom)
            .map(pack)
            .collect();
        // SAFETY: ctor
        unsafe { Self::new(value.width, value.height, buf) }
    }
}

pub fn unpack_all<const N: usize>(buffer: &[u32]) -> impl Iterator<Item = u8> + '_
where
    [u8; N]: PFrom<4>,
{
    buffer
        .iter()
        .copied()
        .map(unpack)
        .flat_map(<[u8; N] as PFrom<4>>::pfrom)
}

impl<const N: usize> From<Image<&[u32], 1>> for Image<Box<[u8]>, N>
where
    [u8; N]: PFrom<4>,
{
    fn from(value: Image<&[u32], 1>) -> Self {
        let buf = unpack_all(value.buffer).collect();
        // SAFETY: ctor
        unsafe { Self::new(value.width, value.height, buf) }
    }
}
