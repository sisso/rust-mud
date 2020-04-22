pub mod geometry;
pub mod strinput;
pub mod text;

/// returns the value between v0 and v1 on t
pub fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    v0 + clamp01(t) * (v1 - v0)
}

/// returns % of t between v0 and v1
pub fn inverse_lerp(v0: f32, v1: f32, t: f32) -> f32 {
    if v0 == v1 {
        0.0
    } else {
        clamp01((t - v0) / (v1 - v0))
    }
}

///
/// Lerp between v0 and v1 giving the value of 5 between t0 and t1
///
/// t <= t0, returns v0
/// t >= t1, returns v1
///
pub fn lerp_2(v0: f32, v1: f32, t0: f32, t1: f32, t: f32) -> f32 {
    let tt = inverse_lerp(t0, t1, t);
    lerp(v0, v1, tt)
}

pub fn clamp01(v: f32) -> f32 {
    if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

#[cfg(test)]
mod test {
    use crate::utils::lerp_2;

    #[test]
    fn test_lerp_2() {
        assert_eq!(lerp_2(0.0, 1.0, 0.0, 1.0, 0.5), 0.5);
        assert_eq!(lerp_2(0.0, 2.0, 0.0, 1.0, 0.5), 1.0);
        assert_eq!(lerp_2(0.0, 1.0, 0.0, 2.0, 1.0), 0.5);
    }
}
