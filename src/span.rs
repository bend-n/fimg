use crate::At;
use std::ops::Range;

mod sealer {
    #[doc(hidden)]
    pub trait Sealed {}
}
use sealer::Sealed;

/// Trait for that which can be used to index a image.
pub trait Span: Sealed {
    #[doc(hidden)]
    fn range<const C: usize>(self, i: (u32, u32)) -> Range<usize>;
}

impl Sealed for Range<usize> {}
impl Span for Range<usize> {
    #[inline(always)]
    fn range<const C: usize>(self, _: (u32, u32)) -> Range<usize> {
        self
    }
}

impl Sealed for Range<(u32, u32)> {}
impl Span for Range<(u32, u32)> {
    #[inline(always)]
    fn range<const C: usize>(self, i: (u32, u32)) -> Range<usize> {
        let Self {
            start: (sx, sy),
            end: (ex, ey),
        } = self;
        i.at::<C>(sx, sy)..i.at::<C>(ex, ey)
    }
}

impl Sealed for (u32, u32) {}
impl Span for (u32, u32) {
    #[inline(always)]
    fn range<const C: usize>(self, i: (u32, u32)) -> Range<usize> {
        i.at::<C>(self.0, self.1)..i.at::<C>(self.0, self.1) + C
    }
}
