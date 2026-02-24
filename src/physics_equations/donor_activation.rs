use crate::constants::physics::*;

#[derive(Debug)]
pub struct DonorActivation {
    temperature: f64,
    q_per_kbt: f64,
}

impl DonorActivation {
    pub fn new(temperature: f64) -> Self {
        Self {
            temperature,
            q_per_kbt: Q_ELECTRON / (K_BOLTZMANN * temperature),
        }
    }

    pub fn get_temperature(&self) -> f64 {
        self.temperature
    }
    /// Ionized donor density
    ///
    /// # Arguments
    ///
    /// - `donor_concentration` (`f64`) - The total donor density in the material.
    /// - `donor_concentration` (`f64`) - The total donor density in the material.
    ///
    /// # Returns
    ///
    /// - `f64` - The ionized donor density in the material.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::physics_equations::donor_activation;
    ///
    /// let _ = donor_activation::ionized_donor_concentration();
    /// ```
    pub fn ionized_donor_concentration(&self, donor_concentration: f64, potential: f64) -> f64 {
        let ion_nd = donor_concentration / (1.0 + 2.0 * (-potential * self.q_per_kbt).exp());
        ion_nd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;
    use test_case::test_case;

    #[test_case(1e22, 300.0, 1.0, 1e22 ; "high-potential")]
    #[test_case(1e22, 300.0, 0.5, 1e22 ; "midium-potential")]
    #[test_case(1e22, 300.0, 0.0, 3.333e21 ; "low-potential")]
    fn test_ionized_donor_concentration(
        donor_concentration: f64,
        temperature: f64,
        potential: f64,
        expected_ionized_donor_concentration: f64,
    ) {
        let donor_activation = DonorActivation::new(temperature);
        let ionized_donor_concentration =
            donor_activation.ionized_donor_concentration(donor_concentration, potential);
        assert!(relative_eq!(
            ionized_donor_concentration,
            expected_ionized_donor_concentration,
            max_relative = 1e-3
        ));
    }
}
