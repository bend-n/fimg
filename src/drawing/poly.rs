//! draw polygons
use std::{
    cmp::{max, min},
    f32::consts::TAU,
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

    /// Draws a filled quadrilateral.
    /// This currently just uses [`Image::points`], but in the future this may change.
    pub fn quad(
        &mut self,
        a: (i32, i32),
        b: (i32, i32),
        c: (i32, i32),
        d: (i32, i32),
        col: [u8; CHANNELS],
    ) {
        self.points(&[a, b, c, d, a], col);
    }

    /// Draws a regular convex polygon with a specified number of sides, a radius, and a rotation (radians).
    /// Prefer [`Image::circle`] over `poly(.., 600, ..)`.
    /// Calls into [`Image::tri`] and [`Image::quad`].
    /// ```
    /// # use fimg::Image;
    /// let mut i = Image::alloc(300, 300);
    /// //          draw a enneagon
    /// // at  x150, y150    │  unrotated   white
    /// // with a radius of ─┼──╮      │      │
    /// i.poly((150., 150.), 9, 100.0, 0.0, [255]);
    /// # assert_eq!(i.buffer(), include_bytes!("../../tdata/enneagon.imgbuf"));
    /// ```
    pub fn poly(
        &mut self,
        (x, y): (f32, f32),
        sides: usize,
        radius: f32,
        rotation: f32,
        c: [u8; CHANNELS],
    ) {
        let trans = |a: f32| (a.cos() * radius, a.sin() * radius);
        let r = |(a, b): (f32, f32)| (a.round() as i32, b.round() as i32);
        let add = |(a, b)| (a + x, b + y);
        match sides {
            3 => {
                let space = TAU / 3.0;
                self.tri(
                    add(trans(space + rotation)),
                    add(trans(rotation)),
                    add(trans(space.mul_add(2.0, rotation))),
                    c,
                );
            }
            _ => {
                let space = TAU / sides as f32;
                for i in (0..sides - 1).step_by(2).map(|i| i as f32) {
                    self.quad(
                        r((x, y)),
                        r(add(trans(space.mul_add(i, rotation)))),
                        r(add(trans(space.mul_add(i + 1., rotation)))),
                        r(add(trans(space.mul_add(i + 2., rotation)))),
                        c,
                    );
                }

                if sides % 2 != 0 && sides > 4 {
                    let i = (sides - 1) as f32;
                    // the missing piece
                    self.tri(
                        (x, y),
                        add(trans(space.mul_add(i, rotation))),
                        add(trans(space.mul_add(i + 1., rotation))),
                        c,
                    );
                }
            }
        }
    }
}
