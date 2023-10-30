//! module for pixel blending ops
use super::{unfloat, Floatify, PMap, Trunc, Unfloatify};
use umath::FF32;

/// Trait for blending pixels together.
pub trait Blend<const W: usize> {
    /// blends self with another pixel
    fn blend(&mut self, with: [u8; W]);
}

impl Blend<4> for [u8; 4] {
    fn blend(&mut self, fg: [u8; 4]) {
        if fg[3] == 0 {
            return;
        }
        if fg[3] == 255 {
            *self = fg;
            return;
        }
        let fg = fg.float();
        let bg = self.float();
        let a = bg[3] + fg[3] - bg[3] * fg[3];
        if a == 0.0 {
            return;
        };
        self[..3].copy_from_slice(
            &fg.trunc()
                // SAFETY: no u8 can possibly become INF / NAN
                .pmap(bg.trunc(), |f, b| unsafe {
                    (f * fg[3] + b * bg[3] * (FF32::new(1.0) - fg[3])) / a
                })
                .unfloat(),
        );
        self[3] = unfloat(a);
    }
}

impl Blend<3> for [u8; 3] {
    fn blend(&mut self, with: [u8; 3]) {
        *self = with;
    }
}

impl Blend<2> for [u8; 2] {
    fn blend(&mut self, with: [u8; 2]) {
        let bg = self.float();
        let fg = with.float();

        let a = bg[1] + fg[1] - bg[1] * fg[1];
        if a == 0.0 {
            return;
        }
        *self = [
            // SAFETY: no u8 can do transform bad
            (fg[0] * fg[1] + bg[0] * bg[1] * (unsafe { FF32::new(1.0) } - fg[1])) / a,
            a,
        ]
        .unfloat();
    }
}

impl Blend<1> for [u8; 1] {
    fn blend(&mut self, with: [u8; 1]) {
        *self = with;
    }
}

#[cfg(test)]
mod blend {
    use super::*;

    macro_rules! blend {
    ([$($a:literal),+] + [$($b:literal),+] = $what:expr) => {
        let mut a = [$($a,)+];
        a.blend([$($b,)+]);
        assert_eq!(a, $what);
    };
}

    #[test]
    fn test_blend_rgba() {
        blend!([255, 255, 255, 255] + [255, 255, 255, 255] = [255, 255, 255, 255]);
        blend!([255, 255, 255, 0] + [255, 255, 255, 255] = [255, 255, 255, 255]);
        blend!([255, 255, 255, 255] + [255_u8, 255, 255, 0] = [255, 255, 255, 255]);
        blend!([255, 255, 255, 0] + [255_u8, 255, 255, 0] = [255, 255, 255, 0]);
    }

    #[test]
    fn test_blend_ya() {
        blend!([255, 255] + [255, 255] = [255, 255]);
        blend!([255, 0] + [255, 255] = [255, 255]);
        blend!([255, 255] + [255, 0] = [255, 255]);
        blend!([255, 0] + [255, 0] = [255, 0]);
    }
}
