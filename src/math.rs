//! utility math
/// Calculates `a * b + c`, with hardware support if possible.
#[allow(clippy::suboptimal_flops)]
pub fn madd(a: f32, b: f32, c: f32) -> f32 {
    if cfg!(target_feature = "fma") {
        a.mul_add(b, c)
    } else {
        a * b + c
    }
}

/// helps
pub trait FExt {
    /// Calculates `a * b + c`, with hardware support if possible.
    fn madd(self, a: f32, b: f32) -> Self;
}

impl FExt for f32 {
    fn madd(self, a: f32, b: f32) -> Self {
        madd(self, a, b)
    }
}
