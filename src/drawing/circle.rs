//! draw 2d circles
use crate::Image;


impl<T: AsMut<[u8]> + AsRef<[u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// Draws a circle, using the [Bresenham's circle](https://en.wikipedia.org/wiki/Midpoint_circle_algorithm) algorithm.
    /// ```
    /// # use fimg::Image;
    /// let mut i = Image::alloc(50, 50);
    /// i.border_circle((25, 25), 20, [255]);
    /// # assert_eq!(i.buffer(), include_bytes!("../../tdata/circle.imgbuf"));
    /// ```
    pub fn border_circle(&mut self, (xc, yc): (i32, i32), radius: i32, c: [u8; CHANNELS]) {
        let mut x = 0;
        let mut y = radius;
        let mut p = 1 - radius;
        /// bounds the pixels
        macro_rules! bound {
            ($($x:expr,$y:expr);+;) => {
                $(if $x >= 0 && $x < self.width() as i32 && $y >= 0 && $y < self.height() as i32 {
                    // SAFETY: ^
                    unsafe { self.set_pixel($x as u32, $y as u32, c) };
                })+
            };
        }
        while x <= y {
            bound! {
                xc + x, yc + y;
                xc + y, yc + x;
                xc - y, yc + x;
                xc - x, yc + y;
                xc - x, yc - y;
                xc - y, yc - x;
                xc + y, yc - x;
                xc + x, yc - y;
            };
            x += 1;
            if p < 0 {
                p += 2 * x + 1;
            } else {
                y -= 1;
                p += 2 * (x - y) + 1;
            }
        }
    }

    /// Draw a filled circle.
    /// ```
    /// # use fimg::Image;
    /// let mut i = Image::alloc(50, 50);
    /// i.circle((25, 25), 20, [255]);
    /// # assert_eq!(i.buffer(), include_bytes!("../../tdata/circle2.imgbuf"));
    /// ```
    pub fn circle(&mut self, (xc, yc): (i32, i32), radius: i32, c: [u8; CHANNELS]) {
        for x in -radius..radius {
            let h = ((radius * radius - x * x) as f32).sqrt().round() as i32;
            for y in -h..h {
                let x = x + xc;
                let y = y + yc;
                if x >= 0 && x < self.width() as i32 && y >= 0 && y < self.height() as i32 {
                    // SAFETY: ^
                    unsafe { self.set_pixel(x as u32, y as u32, c) };
                }
            }
        }
    }
}
