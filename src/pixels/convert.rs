//! provides From's for pixels.
use atools::prelude::*;

/// Converts a pixel to another pixel.
pub trait PFrom<const N: usize> {
    /// Convert a pixel to this pixel.
    fn pfrom(f: [u8; N]) -> Self;
}

impl<const N: usize> PFrom<N> for [u8; N] {
    fn pfrom(f: [u8; N]) -> Self {
        f
    }
}

/// Y pixel
pub type Y = [u8; 1];
impl PFrom<2> for Y {
    fn pfrom(f: YA) -> Self {
        f.trunc()
    }
}

impl PFrom<3> for Y {
    fn pfrom([r, g, b]: RGB) -> Self {
        [((2126 * r as u32 + 7152 * g as u32 + 722 * b as u32) / 10000) as u8]
    }
}

impl PFrom<4> for Y {
    fn pfrom(f: RGBA) -> Self {
        PFrom::pfrom(f.trunc())
    }
}

/// YA pixel
pub type YA = [u8; 2];
impl PFrom<1> for YA {
    fn pfrom(f: Y) -> Self {
        f.join(255)
    }
}

impl PFrom<3> for YA {
    fn pfrom(f: RGB) -> Self {
        Y::pfrom(f).join(255)
    }
}

impl PFrom<4> for YA {
    fn pfrom(f: RGBA) -> Self {
        Y::pfrom(f.trunc()).join(255)
    }
}

/// RGB pixel
pub type RGB = [u8; 3];

impl PFrom<1> for RGB {
    fn pfrom([y]: Y) -> Self {
        [y; 3]
    }
}

impl PFrom<2> for RGB {
    fn pfrom([y, _]: YA) -> Self {
        [y; 3]
    }
}

impl PFrom<4> for RGB {
    fn pfrom(f: RGBA) -> Self {
        f.trunc()
    }
}

/// RGBA pixel
pub type RGBA = [u8; 4];

impl PFrom<1> for RGBA {
    fn pfrom([y]: Y) -> Self {
        [y; 3].join(255)
    }
}

impl PFrom<2> for RGBA {
    fn pfrom([y, a]: YA) -> Self {
        [y; 3].join(a)
    }
}

impl PFrom<3> for RGBA {
    fn pfrom(f: [u8; 3]) -> Self {
        f.join(255)
    }
}
