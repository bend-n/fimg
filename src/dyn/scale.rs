use crate::scale::traits::ScalingAlgorithm;

use super::{e, DynImage};

impl<T: AsMut<[u8]> + AsRef<[u8]>> DynImage<T> {
    /// Scale this image with a given scaling algorithm.
    pub fn scale<A: ScalingAlgorithm>(&mut self, width: u32, height: u32) -> DynImage<Box<[u8]>> {
        e!(self => |i| i.scale::<A>(width, height))
    }
}
