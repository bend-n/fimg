//! text raster

use crate::Image;
use fontdue::{layout::TextStyle, Font};
use umath::FFloat;

impl<T: AsMut<[u8]> + AsRef<[u8]>> Image<T, 4> {
    /// Draw text.
    ///
    /// ```
    /// # use fimg::Image;
    /// let font = fontdue::Font::from_bytes(
    /// &include_bytes!("../../tdata/CascadiaCode.ttf")[..],
    /// 	fontdue::FontSettings {
    /// 		scale: 200.0,
    /// 		..Default::default()
    /// 	},
    /// ).unwrap();
    /// let mut i: Image<_, 4> = Image::alloc(750, 250).boxed();
    /// i.text(50, 10, 200.0, &font, "hello", [0, 0, 0, 255]);
    /// assert_eq!(i.buffer(), include_bytes!("../../tdata/text.imgbuf"));
    /// ```
    pub fn text(&mut self, x: u32, y: u32, size: f32, font: &Font, text: &str, color: [u8; 4]) {
        let mut lay =
            fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
        lay.append(&[font], &TextStyle::new(text, size, 0));
        for glpyh in lay.glyphs() {
            let (metrics, bitmap) = font.rasterize(glpyh.parent, size);
            for i in 0..metrics.width {
                for j in 0..metrics.height {
                    // SAFETY: the rasterizer kinda promises that metrics width and height are in bounds
                    let fg = [color[0], color[1], color[2], unsafe {
                        *bitmap.get_unchecked(j * metrics.width + i)
                    }];
                    let x = x + i as u32 + glpyh.x as u32;
                    if x >= self.width() {
                        continue;
                    }
                    let y = y + j as u32 + glpyh.y as u32;
                    if y >= self.height() {
                        continue;
                    }
                    // SAFETY: we clampin
                    let bg = unsafe { self.pixel_mut(x, y) };

                    blend(bg.try_into().unwrap(), fg);
                }
            }
        }
    }
}

pub fn blend(bg: &mut [u8; 4], fg: [u8; 4]) {
    if fg[3] == 0 {
        return;
    }
    if fg[3] == 255 {
        *bg = fg;
        return;
    }
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    // SAFETY: no u8 can possibly become INF / NAN
    unsafe {
        let max = FFloat::new(255.0);
        let bg_a = FFloat::new(bg[3] as f32) / max;
        let fg_a = FFloat::new(fg[3] as f32) / max;
        let a = bg_a + fg_a - bg_a * fg_a;
        if a == 0.0 {
            return;
        };
        // could turn it into array::map
        *bg = [
            *(max
                * ((((FFloat::new(fg[0] as f32) / max) * fg_a)
                    + ((FFloat::new(bg[0] as f32) / max) * bg_a) * (FFloat::new(1.0) - fg_a))
                    / a)) as u8,
            *(max
                * ((((FFloat::new(fg[1] as f32) / max) * fg_a)
                    + ((FFloat::new(bg[1] as f32) / max) * bg_a) * (FFloat::new(1.0) - fg_a))
                    / a)) as u8,
            *(max
                * ((((FFloat::new(fg[2] as f32) / max) * fg_a)
                    + ((FFloat::new(bg[2] as f32) / max) * bg_a) * (FFloat::new(1.0) - fg_a))
                    / a)) as u8,
            *(max * a) as u8,
        ]
    }
}
