/// find the range of the data for plotting
///
/// # Arguments
///
/// - `data` (`&[f64]`) - Float slice containing the data to find the range of.
///
/// # Returns
///
/// - `(f64, f64)` - A tuple containing the minimum and maximum values in the data slice.
///
/// # Examples
///
/// ```
/// // let data = vec![1.0, -2.5, 5.0, 0.0];
/// // let (min, max) = find_range(&data);
/// // assert_eq!((min, max), (-2.5, 5.0));
/// ```
pub fn find_range(data: &[f64]) -> (f64, f64) {
    let min = data.iter().copied().fold(f64::INFINITY, f64::min);
    let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    if !min.is_finite() || !max.is_finite() {
        panic!("invalid data range");
    }

    (min, max)
}
