//! the houser of uninitialized memory. €$@!0В𴬔!℡
//!
//! contains [`Image`], an uninitialized image.
use crate::{CopyWithinUnchecked, span::Span};
use std::{hint::assert_unchecked, mem::MaybeUninit, num::NonZeroU32, ops::Index};

/// A uninitialized image. Be sure to initialize it!
#[derive(Hash)]
pub struct Image<T: Copy, const CHANNELS: usize> {
    /// Has capacity w * h * c
    buffer: Vec<T>,
    width: NonZeroU32,
    height: NonZeroU32,
}

impl<I: Span, T: Copy, const C: usize> Index<I> for Image<T, C> {
    type Output = [T];

    fn index(&self, index: I) -> &Self::Output {
        &self.buffer()[index.range::<C>((self.width(), self.height()))]
    }
}

impl<T: Copy, const CHANNELS: usize> Image<T, CHANNELS> {
    /// Create a new uninit image. This is not init.
    pub fn new(width: NonZeroU32, height: NonZeroU32) -> Self {
        Self {
            buffer: Vec::with_capacity(width.get() as usize * height.get() as usize * CHANNELS),
            width,
            height,
        }
    }

    /// Write to the image.
    ///
    /// # Safety
    /// index must be in bounds.
    /// data and indexed range must have same len.
    pub unsafe fn write(&mut self, data: &[T], i: impl Span) {
        // SAFETY: caller
        let dat = unsafe { self.slice(i) };
        // SAFETY: caller
        unsafe { assert_unchecked(dat.len() == data.len()) };
        dat.write_copy_of_slice(data);
    }

    /// Slice the image.
    ///
    /// # Safety
    /// index must be in bounds.
    pub unsafe fn slice(&mut self, i: impl Span) -> &mut [MaybeUninit<T>] {
        let range = i.range::<CHANNELS>((self.width(), self.height()));
        // SAFETY: assured
        unsafe { self.buf().get_unchecked_mut(range) }
    }

    /// Copy a range to a position.
    ///
    /// # Safety
    ///
    /// both parts must be in bounds.
    pub unsafe fn copy_within(&mut self, i: impl Span, to: usize) {
        let range = i.range::<CHANNELS>((self.width(), self.height()));
        // SAFETY: copy!
        unsafe { self.buf().copy_within_unchecked(range, to) };
    }

    /// # Safety
    ///
    /// the output index is not guaranteed to be in bounds
    #[inline]
    pub fn at(&self, x: u32, y: u32) -> usize {
        crate::At::at::<CHANNELS>((self.width(), self.height()), x, y)
    }

    #[inline]
    /// get the height as a [`u32`]
    pub const fn height(&self) -> u32 {
        self.height.get()
    }

    #[inline]
    /// get the width as a [`u32`]
    pub const fn width(&self) -> u32 {
        self.width.get()
    }

    #[inline]
    /// create a new image
    ///
    /// # Safety
    ///
    /// does not check that buffer.capacity() == w * h * C
    ///
    /// using this with invalid values may result in future UB
    pub const unsafe fn with_buf(buffer: Vec<T>, width: NonZeroU32, height: NonZeroU32) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    /// consumes the image, returning the image buffer
    pub fn take_buffer(self) -> Vec<T> {
        self.buffer
    }

    /// returns a immutable reference to the backing buffer
    pub fn buffer(&self) -> &[T] {
        &self.buffer
    }

    /// returns a mutable reference to the backing buffer
    pub fn buf(&mut self) -> &mut [MaybeUninit<T>] {
        self.buffer.spare_capacity_mut()
    }

    /// initializes this image, assuming you have done your job
    /// # Safety
    /// requires initialization
    pub unsafe fn init(&mut self) {
        // SAFETY: we have trust for our callers.
        unsafe {
            self.buffer
                .set_len(self.width() as usize * self.height() as usize * CHANNELS)
        };
    }

    /// initializes this image, mapping to a normal [`crate::Image`] type.
    ///
    /// # Safety
    /// UB if you have not init the image
    pub unsafe fn assume_init(mut self) -> crate::Image<Vec<T>, CHANNELS> {
        // SAFETY: its apparently init
        unsafe { self.init() };
        // SAFETY: image all init, good to go
        unsafe { crate::Image::new(self.width, self.height, self.buffer) }
    }
}
