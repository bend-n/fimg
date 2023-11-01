//! trongle drawing
use umath::Float;
use vecto::Vector2;

use crate::Image;
use std::cmp::{max, min};

impl<T: AsMut<[u8]> + AsRef<[u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// Draw a (filled) triangle
    /// ```
    /// # use fimg::*;
    /// let mut a = Image::alloc(10, 10);
    /// // draw a triangle
    /// a.as_mut().tri::<f32>(
    ///   (3.0, 2.0), // point a
    ///   (8.0, 7.0), // point b
    ///   (1.0, 8.0), // point c
    ///   [255] // white
    /// );
    /// # assert_eq!(a.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
    /// ```
    pub fn tri<F: Float<f32>>(
        &mut self,
        a: impl Into<Vector2<F>>,
        b: impl Into<Vector2<F>>,
        c: impl Into<Vector2<F>>,
        col: [u8; CHANNELS],
    ) {
        let Vector2 {
            x: mut x1,
            y: mut y1,
        } = a.into();
        let Vector2 {
            x: mut x2,
            y: mut y2,
        } = b.into();
        let Vector2 { x: x3, y: y3 } = c.into();
        // fix winding
        if (x2 - x1) * (y3 - y1) - (y2 - y1) * (x3 - x1) > 0.0 {
            std::mem::swap(&mut x1, &mut x2);
            std::mem::swap(&mut y1, &mut y2);
        }
        let ymin = max(y1.min(y2).min(y3).take() as u32, 0);
        let ymax = min(y1.max(y2).max(y3).take() as u32, self.height());
        let xmin = max(x1.min(x2).min(x3).take() as u32, 0);
        let xmax = min(x1.max(x2).max(x3).take() as u32, self.width());
        for y in ymin..ymax {
            for x in xmin..xmax {
                // algorithm from https://web.archive.org/web/20050408192410/http://sw-shader.sourceforge.net/rasterizer.html, but im too dumb to implement the faster ones
                if unsafe {
                    (x1 - x2) * (F::new(y as f32) - y1) + (-(y1 - y2) * (F::new(x as f32) - x1))
                        > 0.
                        && (x2 - x3) * (F::new(y as f32) - y2)
                            + (-(y2 - y3) * (F::new(x as f32) - x2))
                            > 0.
                        && (x3 - x1) * (F::new(y as f32) - y3)
                            + (-(y3 - y1) * (F::new(x as f32) - x3))
                            > 0.
                } {
                    // SAFETY: x, y are bounded
                    unsafe { self.set_pixel(x, y, col) };
                }
            }
        }
    }
}
