//! provides a [`ImageCloner`]
//!
//! ```
//! # use fimg::Image;
//! # let i = Image::<_, 1>::alloc(5, 5);
//! unsafe { i.cloner().rot_270() };
//! ```
use crate::Image;

/// A neat way to clone a image.
///
/// Consider it a way to clone->apply a image operation, but better.
/// Please note that some methods may(although none at current) have different safety invariants from their in place counterparts.
pub struct ImageCloner<'a, const C: usize>(Image<&'a [u8], C>);

impl<'a, const C: usize> ImageCloner<'a, C> {
    /// duplicate the inner image.
    pub(crate) fn dup(&self) -> Image<Vec<u8>, C> {
        self.0.to_owned()
    }

    /// Create a [`ImageCloner`] from a <code>[Image]<&\[[u8]\]></code>
    pub const fn from(i: Image<&'a [u8], C>) -> Self {
        Self(i)
    }

    /// Alloc a buffer the right size for use
    pub(crate) fn alloc(&self) -> Image<Vec<u8>, C> {
        Image::alloc(self.width(), self.height())
    }
}

impl<'a, const C: usize> std::ops::Deref for ImageCloner<'a, C> {
    type Target = Image<&'a [u8], C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
