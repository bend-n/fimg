//! adds a `line` function to Image
use crate::Image;
use vecto::Vec2;

impl<T: AsMut<[u8]> + AsRef<[u8]>, const CHANNELS: usize> Image<T, CHANNELS> {
    /// Draw a line from point a to point b.
    ///
    /// Points not in bounds will not be included.
    pub fn line(&mut self, a: (i32, i32), b: (i32, i32), color: [u8; CHANNELS]) {
        clipline::clipline(
            ((a.0 as isize, a.1 as isize), (b.0 as isize, b.1 as isize)),
            (
                (0, 0),
                (self.width() as isize - 1, self.height() as isize - 1),
            ),
            // SAFETY: clipline clips
            |x, y| unsafe { self.set_pixel(x as u32, y as u32, color) },
        );
    }

    /// Draw a thick line from point a to point b.
    /// Prefer [`Image::line`] when possible.
    ///
    /// Points not in bounds will not be included.
    ///
    /// Uses [`Image::quad`].
    /// ```
    /// # use fimg::Image;
    /// let mut i = Image::alloc(10, 10);
    /// i.thick_line((2.0, 2.0), (8.0, 8.0), 2.0, [255]);
    /// # assert_eq!(i.buffer(), b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xff\x00\x00");
    /// ```
    pub fn thick_line(
        &mut self,
        a: impl Into<Vec2>,
        b: impl Into<Vec2>,
        stroke: f32,
        color: [u8; CHANNELS],
    ) {
        let a = a.into();
        let b = b.into();
        let w = (a - b).orthogonal().normalized() * (stroke / 2.0);
        macro_rules! p {
            ($x:expr) => {
                #[allow(clippy::cast_possible_truncation)]
                ($x.x.round() as i32, $x.y.round() as i32)
            };
        }
        // order:
        // v x1 v x2
        // [    ]
        // ^ x3 ^ x4
        self.quad(
            p!(a - w), // x1
            p!(b - w), // x2
            p!(b + w), // x3
            p!(a + w), // x4
            color,
        );
    }
}

#[test]
fn line() {
    let mut a = Image::build(5, 5).alloc();
    a.as_mut().line((0, 1), (6, 4), [255]);
    assert_eq!(
            a.buffer,
            b"\x00\x00\x00\x00\x00\xff\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00\xff\xff\x00\x00\x00\x00\x00"
        )
}
