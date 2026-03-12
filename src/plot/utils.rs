pub fn find_range(data: &[f64]) -> (f64, f64) {
    let min = data.iter().copied().fold(f64::INFINITY, f64::min);
    let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    if !min.is_finite() || !max.is_finite() {
        panic!("invalid data range");
    }

    (min, max)
}
