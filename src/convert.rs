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
