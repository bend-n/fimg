//! trongle drawing

use crate::math::madd;
use std::cmp::{max, min};

use crate::Image;

impl<T: AsMut<[u8]> + AsRef<[u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// Draw a (filled) triangle
    /// ```
    /// # use fimg::*;
    /// let mut a = Image::alloc(10, 10);
    /// // draw a triangle
    /// a.as_mut().tri(
    ///   (3.0, 2.0), // point a
    ///   (8.0, 7.0), // point b
    ///   (1.0, 8.0), // point c
    ///   [255] // white
    /// );
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
        let ymin = max(y1.min(y2).min(y3) as u32, 0);
        let ymax = min(y1.max(y2).max(y3) as u32, self.height());
        let xmin = max(x1.min(x2).min(x3) as u32, 0);
        let xmax = min(x1.max(x2).max(x3) as u32, self.width());
        for y in ymin..ymax {
            for x in xmin..xmax {
                let s = madd(x1 - x3, y as f32 - y3, -(y1 - y2) * (x as f32 - x3));
                let t = madd(x2 - x1, y as f32 - y1, -(y2 - y1) * (x as f32 - x1));

                if (s < 0.0) != (t < 0.0) && s != 0.0 && t != 0.0 {
                    continue;
                }

                let d = madd(x3 - x2, y as f32 - y2, -(y3 - y2) * (x as f32 - x2));
                if d == 0.0 || (d < 0.0) == (s + t <= 0.0) {
                    // SAFETY: x, y are bounded
                    unsafe { self.set_pixel(x, y, c) };
                }
            }
        }
    }
}
