use std::ops::{Range, RangeBounds, RangeFull, RangeInclusive};

use crate::Image;

impl<const CHANNELS: usize, T> Image<T, CHANNELS> {
    /// ```
    /// let i = fimg::Image::<_, 1>::alloc(5, 5);
    /// dbg!(i.bounds((0, 0)..(6, 0)));
    /// panic!();
    /// ```
    pub fn bounds<U>(&self, r: impl PBounds) -> std::ops::Range<usize>
    where
        T: AsRef<[U]>,
    {
        let r = r.bound();
        let start = match r.start_bound() {
            std::ops::Bound::Included(&(x, y)) => self.at(x, y),
            std::ops::Bound::Excluded(&(x, y)) => self.at(x, y) + CHANNELS,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            std::ops::Bound::Included(&(x, y)) => self.at(x, y) + CHANNELS,
            std::ops::Bound::Excluded(&(x, y)) => self.at(x, y),
            std::ops::Bound::Unbounded => self.buffer.as_ref().len(),
        };
        start..end
    }
}
pub trait PBounds {
    fn bound(self) -> impl RangeBounds<(u32, u32)>;
}
impl PBounds for Range<(u32, u32)> {
    fn bound(self) -> impl RangeBounds<(u32, u32)> {
        self
    }
}
impl PBounds for RangeInclusive<(u32, u32)> {
    fn bound(self) -> impl RangeBounds<(u32, u32)> {
        self
    }
}
impl PBounds for (Range<u32>, u32) {
    fn bound(self) -> impl RangeBounds<(u32, u32)> {
        (self.0.start, self.1)..(self.0.end, self.1)
    }
}
impl PBounds for (u32, Range<u32>) {
    fn bound(self) -> impl RangeBounds<(u32, u32)> {
        (self.0, self.1.start)..(self.0, self.1.end)
    }
}
