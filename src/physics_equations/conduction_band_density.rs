use crate::constants::physics::*;

/// Conduction band density
///
/// # Arguments
///
/// - `effective_mass` (`f64`) - The effective mass of the electron in kg.
/// - `temperature` (`f64`) - The temperature in Kelvin.
///
/// # Returns
///
/// - `f64` - The conduction band density in units of m^-3.
///
/// # Examples
///
/// ```ignore
/// use crate::physics_equations::conduction_band_density;
///
/// let _ = conduction_band_density(1.08 * crate::constants::physics::M_ELECTRON, 300.0);
/// ```
pub fn conduction_band_density(effective_mass: f64, temperature: f64) -> f64 {
    let coefficient =
        2.0 * (2.0 * std::f64::consts::PI * effective_mass * K_BOLTZMANN * temperature).powf(1.5);
    let denominator = H_PLANCK_CONSTANT.powf(3.0);

    coefficient / denominator
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::physics::M_ELECTRON;
    use approx::relative_eq;
    use test_case::test_case;

    #[test_case(1.08, 300.0, 2.816486e25 ; "Silicon at 300K")]
    #[test_case(0.20, 300.0, 2.244486e24 ; "GaN at 300K")]
    #[test_case(0.97, 300.0, 2.397339e25 ; "Silicon(vertical) at 300K")]
    fn test_conduction_band_density(
        effective_mass_coefficient: f64,
        temp: f64,
        expected_density: f64,
    ) {
        let density = conduction_band_density(effective_mass_coefficient * M_ELECTRON, temp);
        assert!(relative_eq!(density, expected_density, max_relative = 1e-6));
    }
}
