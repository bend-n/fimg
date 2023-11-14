//! trait for packing pixels

use crate::pixels::convert::{PFrom, RGB, RGBA, Y, YA};

#[inline]
pub const fn pack([r, g, b, a]: [u8; 4]) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[inline]
pub const fn unpack(n: u32) -> [u8; 4] {
    [
        ((n >> 16) & 0xFF) as u8,
        ((n >> 8) & 0xFF) as u8,
        (n & 0xFF) as u8,
        ((n >> 24) & 0xFF) as u8,
    ]
}

/// packs and unpacks this pixel
/// note that `unpack(pack(p))` may not equal `p`
pub trait Pack<P = u32> {
    /// pack this pixel
    fn pack(&self) -> P;
    /// unpacks this pixel
    fn unpack(from: P) -> Self;
}

macro_rules! simple {
    ($p:ident) => {
        impl Pack for $p {
            fn pack(&self) -> u32 {
                pack(PFrom::pfrom(*self))
            }

            fn unpack(from: u32) -> $p {
                PFrom::pfrom(unpack(from))
            }
        }
    };
}
simple!(RGBA);
simple!(RGB);
simple!(YA);
simple!(Y);

impl Pack<u8> for Y {
    fn pack(&self) -> u8 {
        self[0]
    }

    fn unpack(from: u8) -> Self {
        [from]
    }
}
