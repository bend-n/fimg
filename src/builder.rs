//! safe builder for the image
//!
//! does not let you do funny things
use std::marker::PhantomData;

use crate::Image;

impl<B: buf::Buffer, const C: usize> Image<B, C> {
    /// creates a builder
    pub const fn build(w: u32, h: u32) -> Builder<B, C> {
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
impl<B: buf::Buffer, const C: usize> Builder<B, C> {
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
    pub fn buf(self, buffer: B) -> Image<B, C> {
        let len = C as u32 * self.width * self.height;
        assert!(
            buffer.len() as u32 == len,
            "invalid buffer size (expected {len}, got {})",
            buffer.len()
        );
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

/// seals the [`Buffer`] trait
mod buf {
    /// A valid buffer for use in the builder
    pub trait Buffer {
        #[doc(hidden)]
        fn len(&self) -> usize;
    }
    impl<T> Buffer for Vec<T> {
        fn len(&self) -> usize {
            self.len()
        }
    }
    impl<T> Buffer for &[T] {
        fn len(&self) -> usize {
            <[T]>::len(self)
        }
    }
    impl<T> Buffer for &mut [T] {
        fn len(&self) -> usize {
            <[T]>::len(self)
        }
    }
    impl<T, const N: usize> Buffer for [T; N] {
        fn len(&self) -> usize {
            N
        }
    }
    impl<T, const N: usize> Buffer for &[T; N] {
        fn len(&self) -> usize {
            N
        }
    }
    impl<T, const N: usize> Buffer for &mut [T; N] {
        fn len(&self) -> usize {
            N
        }
    }
    impl<T, const N: usize> Buffer for Box<[T; N]> {
        fn len(&self) -> usize {
            N
        }
    }
    impl<T> Buffer for Box<[T]> {
        fn len(&self) -> usize {
            <[T]>::len(self)
        }
    }
}
