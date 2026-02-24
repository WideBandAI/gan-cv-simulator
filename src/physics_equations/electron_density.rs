use crate::constants::physics::*;
use crate::physics_equations::band_density::ConductionBandDensity;
use std::fmt::Debug;

pub trait ElectronDensity: Debug {
    /// Electron density
    ///
    /// # Arguments
    ///
    /// - `&self` (`undefined`)
    /// - `potential` (`f64`) - (Ec - Ef) in eV.
    /// - `mass_electron` (`f64`) - The effective mass of the electron, which is a measure of how the electron behaves in a material.
    ///
    /// # Returns
    ///
    /// - `f64` - The electron density in the conduction band.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::physics_equations::electron_density::ElectronDensity;
    ///
    /// let _ = electron_density();
    /// ```
    fn electron_density(&self, potential: f64, mass_electron: f64) -> f64;
    fn set_temperature(&mut self, temperature: f64);
    fn get_temperature(&self) -> f64;
}

#[derive(Debug)]
pub struct BoltzmannApproximation {
    temperature: f64,
    q_per_kbt: f64,
    conduction_band_density: ConductionBandDensity,
}

impl BoltzmannApproximation {
    pub fn new(temperature: f64) -> Self {
        Self {
            temperature,
            q_per_kbt: Q_ELECTRON / (K_BOLTZMANN * temperature),
            conduction_band_density: ConductionBandDensity::new(temperature),
        }
    }
}

impl ElectronDensity for BoltzmannApproximation {
    fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
        self.q_per_kbt = Q_ELECTRON / (K_BOLTZMANN * temperature);
        self.conduction_band_density.set_temperature(temperature);
    }
    fn get_temperature(&self) -> f64 {
        self.temperature
    }

    /// Electron density
    ///
    /// # Arguments
    ///
    /// - `potential` (`f64`) - The potential difference (Ec - Ef) in eV.
    /// - `mass_electron` (`f64`) - The effective mass of the electron in kg.
    ///
    /// # Returns
    ///
    /// - `f64` - The electron density in the conduction band.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::physics_equations::electron_density::ElectronDensity;
    ///
    /// let _ = electron_density();
    /// ```
    fn electron_density(&self, potential: f64, mass_electron: f64) -> f64 {
        let nc = self
            .conduction_band_density
            .conduction_band_density(mass_electron);
        let n = nc * (-potential * self.q_per_kbt).exp();
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
        let model = BoltzmannApproximation::new(temperature);
        let electron_density =
            model.electron_density(potential, effective_mass_coefficient * M_ELECTRON);
        assert!(relative_eq!(
            electron_density,
            expected_electron_density,
            max_relative = 1e-6
        ));
    }
}
