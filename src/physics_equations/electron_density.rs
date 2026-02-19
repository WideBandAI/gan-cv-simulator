use crate::constants::physics::*;
use crate::physics_equations::band_density::conduction_band_density;

pub trait ElectronDensity {
    /// Electron density
    ///
    /// # Arguments
    ///
    /// - `&self` (`undefined`)
    /// - `potential` (`f64`) - (Ec - Ef) in eV.
    /// - `mass_electron` (`f64`) - The effective mass of the electron, which is a measure of how the electron behaves in a material.
    /// - `temperature` (`f64`) - The temperature of the system, which affects the distribution of electrons and their energy levels.
    ///
    /// # Returns
    ///
    /// - `f64` - The electron density in the conduction band.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = electron_density();
    /// ```
    fn electron_density(&self, potential: f64, mass_electron: f64, temperature: f64) -> f64;
}

pub struct BoltzmannApproximation {}

impl ElectronDensity for BoltzmannApproximation {
    fn electron_density(&self, potential: f64, mass_electron: f64, temperature: f64) -> f64 {
        let nc = conduction_band_density(mass_electron, temperature);
        let n = nc * (-potential * Q_ELECTRON / (K_BOLTZMANN * temperature)).exp();
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;
    use test_case::test_case;

    #[test_case(0.0, 0.2, 300.0, 2.244486e24 ; "equal Nc")]
    #[test_case(1.0, 0.2, 300.0, 3.5633331e7 ; "high potential")]
    #[test_case(-0.5, 0.2, 300.0, 5.633097e32 ; "low potential")]
    fn test_boltzmann_approximation(
        potential: f64,
        effective_mass_coefficient: f64,
        temperature: f64,
        expected_electron_density: f64,
    ) {
        let electron_density = BoltzmannApproximation {}.electron_density(
            potential,
            effective_mass_coefficient * M_ELECTRON,
            temperature,
        );
        assert!(relative_eq!(
            electron_density,
            expected_electron_density,
            max_relative = 1e-6
        ));
    }
}
