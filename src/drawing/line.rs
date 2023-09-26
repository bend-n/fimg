//! adds a `line` function to Image
#![allow(clippy::missing_docs_in_private_items)]
use crate::Image;
use std::{
    iter::Iterator,
    ops::{Deref, DerefMut},
};

/// taken from [bresenham-rs](https://github.com/mbr/bresenham-rs)
pub struct Bresenham {
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
    x1: i32,
    diff: i32,
    octant: Octant,
}

#[derive(Copy, Clone)]
struct Octant(u8);

impl Octant {
    #[inline]
    const fn from_points(start: (i32, i32), end: (i32, i32)) -> Self {
        let mut dx = end.0 - start.0;
        let mut dy = end.1 - start.1;

        let mut octant = 0;

        if dy < 0 {
            dx = -dx;
            dy = -dy;
            octant += 4;
        }

        if dx < 0 {
            let tmp = dx;
            dx = dy;
            dy = -tmp;
            octant += 2;
        }

        if dx < dy {
            octant += 1;
        }

        Self(octant)
    }

    #[inline]
    const fn to_octant0(self, p: (i32, i32)) -> (i32, i32) {
        match self.0 {
            0 => (p.0, p.1),
            1 => (p.1, p.0),
            2 => (p.1, -p.0),
            3 => (-p.0, p.1),
            4 => (-p.0, -p.1),
            5 => (-p.1, -p.0),
            6 => (-p.1, p.0),
            7 => (p.0, -p.1),
            _ => unreachable!(),
        }
    }

    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn from_octant0(self, p: (i32, i32)) -> (i32, i32) {
        match self.0 {
            0 => (p.0, p.1),
            1 => (p.1, p.0),
            2 => (-p.1, p.0),
            3 => (-p.0, p.1),
            4 => (-p.0, -p.1),
            5 => (-p.1, -p.0),
            6 => (p.1, -p.0),
            7 => (p.0, -p.1),
            _ => unreachable!(),
        }
    }
}

impl Bresenham {
    /// Creates a new iterator. Yields intermediate points between `start`
    /// and `end`. Includes `start` and `end`.
    #[inline]
    pub const fn new(start: (i32, i32), end: (i32, i32)) -> Self {
        let octant = Octant::from_points(start, end);

        let start = octant.to_octant0(start);
        let end = octant.to_octant0(end);

        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        Self {
            x: start.0,
            y: start.1,
            dy,
            dx,
            x1: end.0,
            diff: dy - dx,
            octant,
        }
    }
}

impl Iterator for Bresenham {
    type Item = (i32, i32);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x > self.x1 {
            return None;
        }

        let p = (self.x, self.y);

        if self.diff >= 0 {
            self.y += 1;
            self.diff -= self.dx;
        }

        self.diff += self.dy;

        // loop inc
        self.x += 1;

        Some(self.octant.from_octant0(p))
    }
}

impl<T: Deref<Target = [u8]> + DerefMut<Target = [u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// Draw a line from point a to point b
    ///
    /// Points not in bounds will not be included.
    ///
    /// Uses [bresenshams](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm) line algorithm.
    pub fn line(&mut self, a: (i32, i32), b: (i32, i32), color: [u8; CHANNELS]) {
        for (x, y) in Bresenham::new(a, b).map(|(x, y)| (x as u32, y as u32)) {
            if x < self.width() && y < self.height() {
                // SAFETY: bound are checked ^
                unsafe { self.set_pixel(x, y, color) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bresenham() {
        macro_rules! test_bresenham {
            ($a:expr, $b:expr => [$(($x:expr, $y:expr)),+]) => {{
                let mut bi = Bresenham::new($a, $b);
                $(assert_eq!(bi.next(), Some(($x, $y)));)+
                assert_eq!(bi.next(), None);
            }}
        }
        test_bresenham!((6, 4), (0, 1) => [(6, 4), (5, 4), (4, 3), (3, 3), (2, 2), (1, 2), (0, 1)]);
        test_bresenham!((2, 3), (2, 6) => [(2, 3), (2, 4), (2, 5), (2, 6)]);
        test_bresenham!((2, 3), (5, 3) => [(2, 3), (3, 3), (4, 3), (5, 3)]);
        test_bresenham!((0, 1), (6, 4) => [(0, 1), (1, 1), (2, 2), (3, 2), (4, 3), (5, 3), (6, 4)]);
    }

    #[test]
    fn line() {
        let mut a = Image::build(5, 5).alloc();
        a.as_mut().line((0, 1), (6, 4), [255]);
        assert_eq!(
            a.buffer,
            b"\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\xff\x00\x00\x00\x00\x00"
        )
    }
}
