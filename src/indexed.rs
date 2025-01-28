//! indexed images! whoo! (palette and `Image<[u8], 1>`, basically.)
#![allow(private_bounds)]
mod builder;

use crate::Image;

#[allow(non_camel_case_types)]
trait uint: Copy + TryInto<usize> {
    fn nat(self) -> usize {
        self.try_into().ok().unwrap()
    }
}

macro_rules! int {
    ($($t:ty)+) => {
        $(impl uint for $t {})+
    };
}
int!(u8 u16 u32 u64 u128);

/// An image with a palette.
#[derive(Clone)]
pub struct IndexedImage<INDEX, PALETTE> {
    // likely Box<[u8]>, …
    // safety invariant: when INDEX<impl uint>, and PALETTE: Buffer, U must be < len(PALETTE)
    buffer: Image<INDEX, 1>,
    palette: PALETTE, // likely Box<[[f32; 4]]>, …
}

impl<I, P> IndexedImage<I, P> {
    /// Indexes the palette with each index.
    pub fn to<PIXEL: Copy, INDEX: uint, const CHANNELS: usize>(
        &self,
    ) -> Image<Box<[PIXEL]>, CHANNELS>
    where
        P: AsRef<[[PIXEL; CHANNELS]]>,
        I: AsRef<[INDEX]>,
    {
        unsafe {
            self.buffer.map(|x| {
                x.as_ref()
                    .iter()
                    .flat_map(|x| self.palette.as_ref()[x.nat()])
                    .collect()
            })
        }
    }

    /// Gets a mut ref to raw parts.
    pub unsafe fn raw<INDEX: uint>(&mut self) -> Image<&mut [INDEX], 1>
    where
        I: AsMut<[INDEX]>,
    {
        self.buffer.as_mut()
    }

    /// Provides the buffer and palette of this image.
    pub fn into_raw_parts(self) -> (Image<I, 1>, P) {
        (self.buffer, self.palette)
    }

    /// Creates a indexed image from its 1 channel image representation and palette.
    pub fn from_raw_parts<INDEX: uint, PIXEL>(
        buffer: Image<I, 1>,
        palette: P,
    ) -> Result<Self, &'static str>
    where
        I: AsRef<[INDEX]>,
        P: AsRef<[PIXEL]>,
    {
        let good = buffer.chunked().all(|[x]| x.nat() < palette.as_ref().len());
        good.then_some(Self { buffer, palette })
            .ok_or("not all indexes are in palette")
    }
}
