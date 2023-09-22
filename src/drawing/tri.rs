//! trongle drawing
use crate::Image;

impl<const CHANNELS: usize> Image<&mut [u8], CHANNELS> {
    /// Draw a (filled) triangle
    /// ```
    /// # use fimg::*;
    /// let mut a = Image::alloc(10, 10);
    /// // draw a triangle from point a v   point b v    point c v
    /// //                                               with color white
    /// a.as_mut().tri((3.0, 2.0), (8.0, 7.0), (1.0, 8.0), [255]);
    /// # assert_eq!(a.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
    /// ```
    pub fn tri(
        &mut self,
        (x1, y1): (f32, f32),
        (x2, y2): (f32, f32),
        (x3, y3): (f32, f32),
        c: [u8; CHANNELS],
    ) {
        // TODO optimize
        for y in y1.min(y2).min(y3) as u32..y1.max(y2).max(y3) as u32 {
            for x in x1.min(x2).min(x3) as u32..x1.max(x2).max(x3) as u32 {
                let s = (x1 - x3) * (y as f32 - y3) - (y1 - y2) * (x as f32 - x3);
                let t = (x2 - x1) * (y as f32 - y1) - (y2 - y1) * (x as f32 - x1);

                if (s < 0.0) != (t < 0.0) && s != 0.0 && t != 0.0 {
                    continue;
                }

                let d = (x3 - x2) * (y as f32 - y2) - (y3 - y2) * (x as f32 - x2);
                if (d == 0.0 || (d < 0.0) == (s + t <= 0.0))
                    && x < self.width()
                    && y < self.height()
                {
                    // SAFETY: we just checked the bounds
                    unsafe { self.set_pixel(x, y, c) };
                }
            }
        }
    }
}
