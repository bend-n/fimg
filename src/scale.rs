//! holds scaling operations, at current only the Nearest Neighbor
use crate::Image;

/// [Nearest Neighbor](https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation) image scaling algorithm implementation.
/// Use [`Nearest::scale`].
pub struct Nearest;
impl Nearest {
    /// Resize a image.
    /// # Safety
    ///
    /// `image` must be as big or bigger than `width`, `height.
    pub unsafe fn scale<const N: usize>(
        image: Image<&[u8], N>,
        width: u32,
        height: u32,
    ) -> Image<Vec<u8>, N> {
        let x_scale = image.width() as f32 / width as f32;
        let y_scale = image.height() as f32 / height as f32;
        let mut out = Image::alloc(width, height);
        for y in 0..height {
            for x in 0..width {
                let x1 = ((x as f32 + 0.5) * x_scale).floor() as u32;
                let y1 = ((y as f32 + 0.5) * y_scale).floor() as u32;
                // SAFETY: i asked the caller to make sure its ok
                let px = unsafe { image.pixel(x1, y1) };
                // SAFETY: were looping over the width and height of out. its ok.
                unsafe { out.set_pixel(x, y, px) };
            }
        }
        out
    }
}

#[test]
fn test_nearest() {
    let i = Image::<_, 3>::open("src/cat.png");
    assert_eq!(
        unsafe { Nearest::scale(i.as_ref(), 268, 178) }.buffer,
        Image::<_, 3>::open("src/small_cat.png").buffer
    );
}
