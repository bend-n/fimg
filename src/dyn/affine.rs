use super::{e, DynImage};

impl<T: AsMut<[u8]> + AsRef<[u8]>> DynImage<T> {
    /// Rotate this image 90 degrees clockwise.
    ///
    /// # Safety
    ///
    /// UB if this image is not square
    pub unsafe fn rot_90(&mut self) {
        // SAFETY: caller guarantees
        unsafe { e!(self, |i| i.rot_90()) }
    }

    /// Rotate this image 180 degrees clockwise.
    pub fn rot_180(&mut self) {
        e!(self, |i| i.rot_180())
    }

    /// Rotate this image 270 degrees clockwise.
    ///
    /// # Safety
    ///
    /// UB if this image is not square
    pub unsafe fn rot_270(&mut self) {
        // SAFETY: caller guarantees
        unsafe { e!(self, |i| i.rot_270()) }
    }

    /// Flip this image horizontally.
    pub fn flip_h(&mut self) {
        e!(self, |i| i.flip_h())
    }

    /// Flip this image vertically.
    pub fn flip_v(&mut self) {
        e!(self, |i| i.flip_v())
    }
}
