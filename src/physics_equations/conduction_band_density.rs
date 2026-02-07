use crate::constants::physics::*;

/// Conduction band density
///
/// # Arguments
///
/// - `effective_mass_coefficient` (`f64`) - The effective mass coefficient of the material in units of electron mass.
/// - `temperature` (`f64`) - The temperature in Kelvin.
///
/// # Returns
///
/// - `f64` - The conduction band density in units of m^-3.
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let _ = conduction_band_density();
/// ```
pub fn conduction_band_density(effective_mass_coefficient: f64, temperature: f64) -> f64 {
    let coefficient = 2.0
        * (2.0
            * std::f64::consts::PI
            * effective_mass_coefficient
            * M_ELECTRON
            * K_BOLTZMANN
            * temperature)
            .powf(1.5);
    let denominator = H_PLANK_CONSTANT.powf(3.0);

    coefficient / denominator
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;
    use test_case::test_case;

    #[test_case(1.08, 300.0, 2.81e25 ; "Silicon at 300K")]
    #[test_case(0.26, 300.0, 6.02e24 ; "GaAs at 300K")]
    fn test_conduction_band_density(
        effective_mass_coefficient: f64,
        temp: f64,
        expected_density: f64,
    ) {
        let density = conduction_band_density(effective_mass_coefficient, temp);
        let _ = relative_eq!(density, expected_density, max_relative = 1e-3);
    }
}
