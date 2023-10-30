use umath::{generic_float::Constructors, FF32};

use super::{float, unfloat, PMap};

pub trait Wam {
    /// this function weighs the sides and combines
    ///
    /// # Safety
    ///
    /// pls make l = 0..=f32::MAX/2, r = 0..=f32::MAX/2
    unsafe fn wam(self, b: Self, l: FF32, r: FF32) -> Self;
}

impl<const N: usize> Wam for [u8; N] {
    unsafe fn wam(self, b: Self, l: FF32, r: FF32) -> Self {
        // SAFETY: read [`weigh`]
        self.pmap(b, |a, b| unsafe { weigh(a, b, l, r) })
    }
}

#[inline(always)]
/// # Safety
///
/// floats must be smart
unsafe fn weigh(a: u8, b: u8, l: FF32, r: FF32) -> u8 {
    // SAFETY: float(x) returns 0..=1, 0..=1 * f32::MAX isnt Inf, but if you add 1.0 and then mul by max again, you get inf (big bad, hence unsafe fn)
    unsafe { unfloat((float(a) * l + float(b) * r).clamp(FF32::zero(), FF32::one())) }
}

#[test]
fn weig() {
    unsafe {
        assert_eq!(weigh(10, 20, FF32::new(0.5), FF32::new(0.5)), 15);
        assert_eq!(weigh(10, 20, FF32::new(0.9), FF32::new(0.1)), 11);
        assert_eq!(weigh(150, 150, FF32::new(1.8), FF32::new(0.8)), 255);
    }
}
