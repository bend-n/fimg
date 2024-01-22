//! the houser of uninitialized memory. €$@!0В𴬔!℡
//!
//! contains [`Image`], an uninitialized image.
use std::{mem::MaybeUninit, num::NonZeroU32};

use crate::CopyWithinUnchecked;

/// A uninitialized image. Be sure to initialize it!
pub struct Image<T: Copy, const CHANNELS: usize> {
    /// Has capacity w * h * c
    buffer: Vec<T>,
    width: NonZeroU32,
    height: NonZeroU32,
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
    pub unsafe fn write(&mut self, data: &[T], i: impl crate::span::Span) {
        let range = i.range::<CHANNELS>((self.width(), self.height()));
        // SAFETY: write
        let dat = unsafe { self.buf().get_unchecked_mut(range) };
        MaybeUninit::write_slice(dat, data);
    }

    /// Copy a range to a position.
    ///
    /// # Safety
    ///
    /// both parts must be in bounds.
    pub unsafe fn copy_within(&mut self, i: impl crate::span::Span, to: usize) {
        let range = i.range::<CHANNELS>((self.width(), self.height()));
        // SAFETY: copy!
        unsafe { self.buf().copy_within_unchecked(range, to) };
    }

    /// # Safety
    ///
    /// the output index is not guranteed to be in bounds
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
