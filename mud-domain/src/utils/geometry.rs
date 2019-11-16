use commons::V2;

/// returns the new position and true if hit the target
pub fn move_towards(from: V2, to: V2, max_distance: f32) -> (V2, bool) {
    let delta   = to.sub(&from);
    // delta == zero can cause length sqr NaN
    let length_sqr = delta.length_sqr();
    if length_sqr.is_nan() || length_sqr <= max_distance {
        (to, true)
    } else {
        let norm = delta.div(length_sqr.sqrt());
        let mov = norm.mult(max_distance);
        let new_position = from.add(&mov);
        (new_position, false)
    }
}
