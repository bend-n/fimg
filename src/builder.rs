//! safe builder for the image
//!
//! does not let you do funny things
use std::marker::PhantomData;

use crate::Image;

impl<B, const C: usize> Image<B, C> {
    /// creates a builder
    pub const fn build<I>(w: u32, h: u32) -> Builder<B, C>
    where
        B: AsRef<[I]>,
    {
        Builder::new(w, h)
    }
}

/// Safe [`Image`] builder.
#[must_use = "builder must be consumed"]
pub struct Builder<B, const C: usize> {
    /// the width in a zeroable type. zeroable so as to make the check in [`buf`] easier.
    width: u32,
    /// the height in a zeroable type.
    height: u32,
    #[allow(clippy::missing_docs_in_private_items)]
    _buffer: PhantomData<B>,
}
impl<B, const C: usize> Builder<B, C> {
    /// create new builder
    pub const fn new(w: u32, h: u32) -> Self {
        Self {
            width: w,
            height: h,
            _buffer: PhantomData,
        }
    }

    /// apply a buffer, and build
    #[track_caller]
    #[must_use = "what is it going to do?"]
    pub fn buf<I>(self, buffer: B) -> Image<B, C>
    where
        B: AsRef<[I]>,
    {
        let len = C as u32 * self.width * self.height;
        assert!(
            buffer.as_ref().len() as u32 == len,
            "invalid buffer size (expected {len}, got {})",
            buffer.as_ref().len()
        );
        // SAFETY: checked!
        unsafe { self.buf_unchecked(buffer) }
    }
    /// apply a buffer, and build (length unchecked)
    #[track_caller]
    #[must_use = "what is it going to do?"]
    pub unsafe fn buf_unchecked<I>(self, buffer: B) -> Image<B, C>
    where
        B: AsRef<[I]>,
    {
        Image {
            buffer,
            width: self.width.try_into().expect("passed zero width to builder"),
            height: self
                .height
                .try_into()
                .expect("passed zero height to builder"),
        }
    }
}

impl<const C: usize> Builder<Vec<u8>, C> {
    /// allocate this image
    #[must_use = "what is it going to do?"]
    pub fn alloc(self) -> Image<Vec<u8>, C> {
        Image::alloc(self.width, self.height)
    }
}

impl<T: Copy, const C: usize> Builder<Box<[T]>, C> {
    /// Fill this image with a certain pixel.
    /// ```
    /// # use fimg::Image;
    ///
    /// // fill black
    /// Image::build(50, 50).fill([0, 0, 0, 255]);
    /// ```
    pub fn fill(self, with: [T; C]) -> Image<Box<[T]>, C> {
        Image::build(self.width, self.height)
            .buf((0..self.width * self.height).flat_map(|_| with).collect())
    }
}
