use umath::FF32;
pub trait Unfloatify<const N: usize> {
    /// computes 255 * n, for all elements
    fn unfloat(self) -> [u8; N];
}

#[inline(always)]
/// computes 255 * n
pub fn unfloat(n: FF32) -> u8 {
    // SAFETY: n is 0..=1
    unsafe { *(FF32::new(255.0) * n) as u8 }
}

impl<const N: usize> Unfloatify<N> for [FF32; N] {
    fn unfloat(self) -> [u8; N] {
        self.map(unfloat)
    }
}

#[rustfmt::skip]
impl<const N:usize>Unfloatify<N>for[u8; N]{fn unfloat(self)->[u8;N]{self}}

pub trait Floatify<const N: usize> {
    /// computes n / 255, for all elements
    fn float(self) -> [FF32; N];
}

/// computes n / 255
pub fn float(n: u8) -> FF32 {
    // SAFETY: 0..=255 / 0..=255 mayn't ever be NAN / INF
    unsafe { FF32::new(n as f32) / FF32::new(255.0) }
}

impl<const N: usize> Floatify<N> for [u8; N] {
    fn float(self) -> [FF32; N] {
        self.map(float)
    }
}

#[rustfmt::skip]
impl<const N:usize>Floatify<N>for[FF32;N]{fn float(self)->[FF32;N]{self}}
