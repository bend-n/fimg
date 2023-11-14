//! define From's for images.
//! these conversions are defined by [`PFrom`].
use crate::{pixels::convert::PFrom, Image, Pack};

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
