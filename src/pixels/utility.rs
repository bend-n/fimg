#[lower::apply(algebraic)]
pub trait Unfloatify<const N: usize> {
    /// computes 255 * n, for all elements
    fn unfloat(self) -> [u8; N];
}

#[inline(always)]
/// computes 255 * n
pub fn unfloat(n: f32) -> u8 {
    // SAFETY: n is 0..=1
    (255.0 * n) as u8
}

impl<const N: usize> Unfloatify<N> for [f32; N] {
    fn unfloat(self) -> [u8; N] {
        self.map(unfloat)
    }
}

#[rustfmt::skip]
impl<const N:usize>Unfloatify<N>for[u8; N]{fn unfloat(self)->[u8;N]{self}}

pub trait Floatify<const N: usize> {
    /// computes n / 255, for all elements
    fn float(self) -> [f32; N];
}

/// computes n / 255
#[lower::apply(algebraic)]
pub fn float(n: u8) -> f32 {
    // SAFETY: 0..=255 / 0..=255 mayn't ever be NAN / INF
    n as f32 / 255.0
}

impl<const N: usize> Floatify<N> for [u8; N] {
    fn float(self) -> [f32; N] {
        self.map(float)
    }
}

#[rustfmt::skip]
impl<const N:usize>Floatify<N>for[f32;N]{fn float(self)->[f32;N]{self}}
