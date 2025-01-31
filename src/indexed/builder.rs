use super::{uint, IndexedImage as Image};
use std::{marker::PhantomData, mem::MaybeUninit};
impl<B, P> Image<B, P> {
    /// creates a builder
    pub const fn build<I, J>(w: u32, h: u32) -> Builder<B, P>
    where
        B: AsRef<[I]>,
        P: AsRef<[J]>,
    {
        Builder::new(w, h)
    }
}

/// Safe [`Image`] builder.
#[must_use = "builder must be consumed"]
pub struct Builder<B, P> {
    /// the width in a zeroable type. zeroable so as to make the check in [`buf`] easier.
    width: u32,
    /// the height in a zeroable type.
    height: u32,
    palette: Option<P>,
    #[allow(clippy::missing_docs_in_private_items)]
    _buffer: PhantomData<B>,
}
impl<B, P> Builder<B, P> {
    /// create new builder
    pub const fn new(w: u32, h: u32) -> Self {
        Self {
            width: w,
            height: h,
            _buffer: PhantomData,
            palette: None,
        }
    }

    /// add a palette
    pub fn pal(self, p: P) -> Self {
        Self {
            palette: Some(p),
            ..self
        }
    }

    /// apply a buffer, and build
    #[track_caller]
    #[must_use = "what is it going to do?"]
    #[allow(private_bounds)]
    pub fn buf<I: uint, J>(self, buffer: B) -> Image<B, P>
    where
        B: AsRef<[I]>,
        P: AsRef<[J]>,
    {
        Image::from_raw_parts(
            crate::Image::build(self.width, self.height).buf(buffer),
            self.palette.expect("require palette"),
        )
        .unwrap()
    }

    /// Allocates a zeroed buffer.
    #[track_caller]
    #[must_use]
    pub fn alloc<I: uint, J>(self) -> Image<Box<[I]>, P>
    where
        P: AsRef<[J]>,
    {
        let palette = self.palette.expect("require palette");
        assert!(palette.as_ref().len() != 0, "need some palette");
        Image {
            buffer: crate::Image::build(self.width, self.height).buf(
                vec![I::default(); self.width as usize * self.height as usize].into_boxed_slice(),
            ),
            palette,
        }
    }

    pub fn uninit<I: uint, J>(self) -> Image<Box<[MaybeUninit<I>]>, P>
    where
        P: AsRef<[J]>,
    {
        Image {
            buffer: crate::Image::build(self.width, self.height).buf(Box::new_uninit_slice(
                self.width as usize * self.height as usize,
            )),
            palette: self.palette.expect("require palette"),
        }
    }
}
