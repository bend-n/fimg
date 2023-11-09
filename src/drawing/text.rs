//! text raster

use crate::{
    convert::{pack, unpack},
    pixels::{float, Wam},
    Image,
};
use fontdue::{layout::TextStyle, Font};
use umath::{generic_float::Constructors, FF32};

impl Image<&mut [u32], 1> {
    pub(crate) fn text_u32(
        &mut self,
        x: u32,
        y: u32,
        size: f32,
        font: &Font,
        text: &str,
        color: [u8; 4],
    ) {
        let mut lay =
            fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
        lay.append(&[font], &TextStyle::new(text, size, 0));
        for glpyh in lay.glyphs() {
            let (metrics, bitmap) = font.rasterize(glpyh.parent, size);
            for i in 0..metrics.width {
                for j in 0..metrics.height {
                    let x = x + i as u32 + glpyh.x as u32;
                    if x >= self.width() {
                        continue;
                    }
                    let y = y + j as u32 + glpyh.y as u32;
                    if y >= self.height() {
                        continue;
                    }

                    // SAFETY: the rasterizer kinda promises that metrics width and height are in bounds
                    let fill = unsafe { float(*bitmap.get_unchecked(j * metrics.width + i)) };
                    // SAFETY: we clampin
                    let bg = unsafe { unpack(*self.buffer.get_unchecked(self.at(x, y))) };
                    // SAFETY: see above
                    *unsafe { self.buffer.get_unchecked_mut(self.at(x, y)) } =
                        // SAFETY: fill is 0..=1
                        pack(unsafe { bg.wam(color, FF32::one() - fill, fill) });
                }
            }
        }
    }
}

impl<const N: usize, T: AsMut<[u8]> + AsRef<[u8]>> Image<T, N> {
    /// Draw text.
    ///
    /// ```
    /// # use fimg::Image;
    /// let font = fontdue::Font::from_bytes(
    ///     &include_bytes!("../../data/CascadiaCode.ttf")[..],
    ///     fontdue::FontSettings {
    ///         scale: 200.0,
    ///         ..Default::default()
    ///     },
    /// ).unwrap();
    /// let mut i: Image<_, 4> = Image::alloc(750, 250).boxed();
    /// i.text(50, 10, 200.0, &font, "hello", [0, 0, 0, 255]);
    /// # assert_eq!(&**i.buffer(), include_bytes!("../../tdata/text.imgbuf"));
    /// ```
    pub fn text(&mut self, x: u32, y: u32, size: f32, font: &Font, text: &str, color: [u8; N]) {
        let mut lay =
            fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
        lay.append(&[font], &TextStyle::new(text, size, 0));
        for glpyh in lay.glyphs() {
            let (metrics, bitmap) = font.rasterize(glpyh.parent, size);
            for i in 0..metrics.width {
                for j in 0..metrics.height {
                    let x = x + i as u32 + glpyh.x as u32;
                    if x >= self.width() {
                        continue;
                    }
                    let y = y + j as u32 + glpyh.y as u32;
                    if y >= self.height() {
                        continue;
                    }

                    // SAFETY: the rasterizer kinda promises that metrics width and height are in bounds
                    let fill = unsafe { float(*bitmap.get_unchecked(j * metrics.width + i)) };
                    // SAFETY: we clampin
                    let bg = unsafe { &mut *(self.pixel_mut(x, y).as_mut_ptr() as *mut [u8; N]) };
                    // SAFETY: fill is 0..=1
                    *bg = unsafe { bg.wam(color, FF32::one() - fill, fill) };
                }
            }
        }
    }
}
