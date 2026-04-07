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

    pub fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
        self.q_per_kbt = Q_ELECTRON / (K_BOLTZMANN * temperature);
    }

    pub fn get_temperature(&self) -> f64 {
        self.temperature
    }
    /// Ionized donor density
    ///
    /// # Arguments
    ///
    /// - `donor_concentration` (`f64`) - The total donor density in the material.
    /// - `potential` (`f64`) - Ed - Ef in eV. Ed is the donor energy level and Ef is the Fermi level.
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

    /// Derivative of ionized donor density with respect to potential phi (Ed - Ef in eV).
    ///
    /// Formula: d(Nd+)/dphi = Nd * x * (q/kBT) / (1 + x)^2
    /// where x = 2 * exp(-phi * q/kBT)
    pub fn ionized_donor_dphi(&self, donor_concentration: f64, phi: f64) -> f64 {
        let x = 2.0 * (-phi * self.q_per_kbt).exp();
        let denom = 1.0 + x;
        donor_concentration * x * self.q_per_kbt / (denom * denom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;
    use test_case::test_case;

    /// ionized_donor_dphi のテスト：有限差分との比較
    #[test]
    fn test_ionized_donor_dphi_matches_finite_difference() {
        let da = DonorActivation::new(300.0);
        let nd = 1e22_f64;
        let phi = 0.3_f64;
        let eps = 1e-7;
        let numerical = (da.ionized_donor_concentration(nd, phi + eps)
            - da.ionized_donor_concentration(nd, phi - eps))
            / (2.0 * eps);
        let analytical = da.ionized_donor_dphi(nd, phi);
        assert!(
            approx::relative_eq!(analytical, numerical, max_relative = 1e-4),
            "analytical={analytical}, numerical={numerical}"
        );
    }

    /// phi=0 でのチェック（x=2, 分子=Nd*2*(q/kBT)/9）
    #[test]
    fn test_ionized_donor_dphi_at_zero_phi() {
        use crate::constants::physics::{K_BOLTZMANN, Q_ELECTRON};
        let temp = 300.0;
        let da = DonorActivation::new(temp);
        let nd = 1e22_f64;
        let q_per_kbt = Q_ELECTRON / (K_BOLTZMANN * temp);
        // x=2, (1+x)^2=9
        let expected = nd * 2.0 * q_per_kbt / 9.0;
        let result = da.ionized_donor_dphi(nd, 0.0);
        assert!(
            approx::relative_eq!(result, expected, max_relative = 1e-10),
            "result={result}, expected={expected}"
        );
    }

    /// phi が大きい（完全電離域）では微分がほぼ0になること
    #[test]
    fn test_ionized_donor_dphi_large_phi_near_zero() {
        let da = DonorActivation::new(300.0);
        let result = da.ionized_donor_dphi(1e22, 5.0);
        assert!(
            result.abs() < 1e10,
            "should be near zero at large phi: {result}"
        );
    }

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
