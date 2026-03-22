use crate::constants::physics::*;

#[derive(Debug)]
pub struct ConductionBandDensity {
    temperature: f64,
    kbt: f64,
    h_planck_constant_pow3: f64,
}

impl ConductionBandDensity {
    pub fn new(temperature: f64) -> Self {
        Self {
            temperature,
            kbt: K_BOLTZMANN * temperature,
            h_planck_constant_pow3: H_PLANCK_CONSTANT.powf(3.0),
        }
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
        self.kbt = K_BOLTZMANN * temperature;
    }

    pub fn get_temperature(&self) -> f64 {
        self.temperature
    }

    /// Calculate the conduction band density.
    ///
    /// # Arguments
    ///
    /// - `mass_electron` (`f64`) - The effective mass of electron in kg.
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
    pub fn conduction_band_density(&self, mass_electron: f64) -> f64 {
        let coefficient = 2.0 * (2.0 * std::f64::consts::PI * mass_electron * self.kbt).powf(1.5);
        coefficient / self.h_planck_constant_pow3
    }
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
        let density = ConductionBandDensity::new(temp)
            .conduction_band_density(effective_mass_coefficient * M_ELECTRON);
        assert!(relative_eq!(density, expected_density, max_relative = 1e-6));
    }
}
