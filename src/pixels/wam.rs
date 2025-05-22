use super::{float, unfloat};
use atools::prelude::*;
#[allow(dead_code)]
pub trait Wam {
    /// this function weighs the sides and combines
    fn wam(self, b: Self, l: f32, r: f32) -> Self;
}

impl<const N: usize> Wam for [u8; N] {
    fn wam(self, b: Self, l: f32, r: f32) -> Self {
        // SAFETY: read [`weigh`]
        self.zip(b).map(|(a, b)| weigh(a, b, l, r))
    }
}

#[inline(always)]
#[lower::apply(algebraic)]
fn weigh(a: u8, b: u8, l: f32, r: f32) -> u8 {
    unfloat((float(a) * l + float(b) * r).clamp(0., 1.))
}

#[test]
fn weig() {
    assert_eq!(weigh(10, 20, 0.5, 0.5), 15);
    assert_eq!(weigh(10, 20, 0.9, 0.1), 11);
    assert_eq!(weigh(150, 150, 1.8, 0.8), 255);
}
