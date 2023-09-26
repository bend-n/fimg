//! draw polygons
use std::{
    cmp::{max, min},
    ops::DerefMut,
};

use crate::Image;

impl<T: DerefMut<Target = [u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// Draws a filled polygon from a slice of points. Please close your poly. (first == last)
    ///
    /// Borrowed from [imageproc](https://docs.rs/imageproc/latest/src/imageproc/drawing/polygon.rs.html#31), modified for less allocations.
    /// ```
    /// # use fimg::Image;
    /// let mut i = Image::alloc(10, 10);
    /// i.points(&[(1, 8), (3, 1), (8, 1), (6, 6), (8, 8), (1, 8)], [255]);
    /// # assert_eq!(i.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\xff\xff\xff\xff\xff\xff\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
    /// ```
    pub fn points(&mut self, poly: &[(i32, i32)], c: [u8; CHANNELS]) {
        if poly.len() <= 1 {
            return;
        }
        let (mut y_max, mut y_min) = poly[..poly.len() - 1]
            .iter()
            .fold((i32::MIN, i32::MAX), |(max, min), &(_, y)| {
                (y.max(max), y.min(min))
            });
        y_min = max(0, min(y_min, self.height() as i32 - 1));
        y_max = max(0, min(y_max, self.height() as i32 - 1));
        let mut intersections = vec![];
        for y in y_min..=y_max {
            for [p0, p1] in poly.array_windows::<2>() {
                if p0.1 <= y && p1.1 >= y || p1.1 <= y && p0.1 >= y {
                    if p0.1 == p1.1 {
                        intersections.push(p0.0);
                        intersections.push(p1.0);
                    } else if p0.1 == y || p1.1 == y {
                        if p1.1 > y {
                            intersections.push(p0.0);
                        }
                        if p0.1 > y {
                            intersections.push(p1.0);
                        }
                    } else {
                        let fraction = (y - p0.1) as f32 / (p1.1 - p0.1) as f32;
                        let inter = fraction.mul_add((p1.0 - p0.0) as f32, p0.0 as f32);
                        intersections.push(inter.round() as i32);
                    }
                }
            }
            intersections.sort_unstable();
            for &[x, y_] in intersections.array_chunks::<2>() {
                let mut from = min(x, self.width() as i32);
                let mut to = min(y_, self.width() as i32 - 1);
                if from < self.width() as i32 && to >= 0 {
                    // check bounds
                    from = max(0, from);
                    to = max(0, to);

                    for x in from..=to {
                        // SAFETY: bounds are checked
                        unsafe { self.set_pixel(x as u32, y as u32, c) };
                    }
                }
            }
            intersections.clear();
        }

        for &[(x1, y1), (x2, y2)] in poly.array_windows::<2>() {
            self.line((x1, y1), (x2, y2), c);
        }
    }
}
