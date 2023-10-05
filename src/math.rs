/// Calculates `a * b + c`, with hardware support if possible.
pub fn madd(a: f32, b: f32, c: f32) -> f32 {
    if cfg!(target_feature = "fma") {
        a.mul_add(b, c)
    } else {
        a * b + c
    }
}
