use crate::constants::physics::*;
use crate::physics_equations::band_density::ConductionBandDensity;

#[derive(Debug)]
pub struct SRHStatics {
    temperature: f64,
    q_per_kbt: f64,
    thermal_velocity: f64,
    mass_electron: f64,
    conduction_band_density: f64,
}

/// SRH statics
///
/// # Arguments
///
/// - `temperature` (`f64`) - temperature in K.
/// - `mass_electron` (`f64`) - mass of electron in kg.
/// - `thermal_velocity` (`f64`) - thermal velocity.
///
/// # Returns
///
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let srh = RHStatics::new(temp, GAN_MASS_COEFF * M_ELECTRON, THERMAL_VELOCITY)
/// ```
impl SRHStatics {
    pub fn new(temperature: f64, mass_electron: f64, thermal_velocity: f64) -> Self {
        let conduction_band_density =
            ConductionBandDensity::new(temperature).conduction_band_density(mass_electron);
        Self {
            temperature,
            q_per_kbt: Q_ELECTRON / (K_BOLTZMANN * temperature),
            thermal_velocity,
            mass_electron,
            conduction_band_density,
        }
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
        self.q_per_kbt = Q_ELECTRON / (K_BOLTZMANN * temperature);
        self.conduction_band_density =
            ConductionBandDensity::new(temperature).conduction_band_density(self.mass_electron);
    }

    pub fn get_temperature(&self) -> f64 {
        self.temperature
    }

    pub fn set_thermal_velocity(&mut self, thermal_velocity: f64) {
        self.thermal_velocity = thermal_velocity;
    }

    pub fn get_thermal_velocity(&self) -> f64 {
        self.thermal_velocity
    }

    pub fn set_mass_electron(&mut self, mass_electron: f64) {
        self.conduction_band_density =
            ConductionBandDensity::new(self.temperature).conduction_band_density(mass_electron);
    }

    /// Electron emission time in sec
    ///
    /// # Arguments
    ///
    /// - `potential` (`f64`) - Ec - Et in eV. Et is the trap energy level.
    /// - `capture_cross_section` (`f64`) - capture cross-section in m².
    ///
    /// # Returns
    ///
    /// - `f64` - electron emission time in sec.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = electron_emission_time();
    /// ```
    pub fn electron_emission_time(&self, potential: f64, capture_cross_section: f64) -> f64 {
        (potential * self.q_per_kbt).exp()
            / (self.thermal_velocity * capture_cross_section * self.conduction_band_density)
    }

    /// Effective emission coefficient
    ///
    /// # Arguments
    ///
    /// - `time` (`f64`) - time in sec.
    /// - `potential` (`f64`) - Ec - Et in eV. Et is the trap energy level.
    /// - `capture_cross_section` (`f64`) - capture cross-section in m².
    ///
    /// # Returns
    ///
    /// - `f64` - effective emission coefficient.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = effective_emission_coefficient();
    /// ```
    pub fn effective_emission_coefficient(
        &self,
        time: f64,
        potential: f64,
        capture_cross_section: f64,
    ) -> f64 {
        let tau = self.electron_emission_time(potential, capture_cross_section);
        1.0 - (-time / tau).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::physics::{K_BOLTZMANN, M_ELECTRON, Q_ELECTRON};
    use approx::relative_eq;
    use test_case::test_case;

    const THERMAL_VELOCITY: f64 = 2.6e5; // m/s, typical for GaN
    const GAN_MASS_COEFF: f64 = 0.20;

    fn make_srh(temp: f64) -> SRHStatics {
        SRHStatics::new(temp, GAN_MASS_COEFF * M_ELECTRON, THERMAL_VELOCITY)
    }

    #[test]
    fn test_new_temperature() {
        let srh = make_srh(300.0);
        assert_eq!(srh.get_temperature(), 300.0);
    }

    #[test]
    fn test_new_thermal_velocity() {
        let srh = make_srh(300.0);
        assert_eq!(srh.get_thermal_velocity(), THERMAL_VELOCITY);
    }

    #[test]
    fn test_set_temperature() {
        let mut srh = make_srh(300.0);
        srh.set_temperature(400.0);
        assert_eq!(srh.get_temperature(), 400.0);
    }

    #[test]
    fn test_set_thermal_velocity() {
        let mut srh = make_srh(300.0);
        srh.set_thermal_velocity(3.0e5);
        assert_eq!(srh.get_thermal_velocity(), 3.0e5);
    }

    // potential=0 のとき exp 項が 1 になるので tau = 1 / (v_th * sigma * Nc)
    // Nc(GaN, 300K) = 2.244486e24 m^-3 (band_density のテストより)
    #[test]
    fn test_electron_emission_time_at_zero_potential() {
        let srh = make_srh(300.0);
        let sigma = 1e-15;
        let tau = srh.electron_emission_time(0.0, sigma);
        let nc_gan_300k = 2.244486e24_f64;
        let expected = 1.0 / (THERMAL_VELOCITY * sigma * nc_gan_300k);
        assert!(relative_eq!(tau, expected, max_relative = 1e-5));
    }

    // tau の比は exp((E2 - E1) * q/kT) に等しい
    #[test_case(300.0, 0.0, 0.1 ; "300K from 0.0 to 0.1 eV")]
    #[test_case(300.0, 0.1, 0.5 ; "300K from 0.1 to 0.5 eV")]
    #[test_case(400.0, 0.0, 0.3 ; "400K from 0.0 to 0.3 eV")]
    fn test_electron_emission_time_ratio(temp: f64, e1: f64, e2: f64) {
        let srh = make_srh(temp);
        let sigma = 1e-15;
        let tau1 = srh.electron_emission_time(e1, sigma);
        let tau2 = srh.electron_emission_time(e2, sigma);
        let ratio = tau2 / tau1;
        let q_per_kbt = Q_ELECTRON / (K_BOLTZMANN * temp);
        let expected_ratio = ((e2 - e1) * q_per_kbt).exp();
        assert!(relative_eq!(ratio, expected_ratio, max_relative = 1e-10));
    }

    // t=0 のとき det = 1 - exp(0) = 0
    #[test]
    fn test_effective_emission_coefficient_at_zero_time() {
        let srh = make_srh(300.0);
        let det = srh.effective_emission_coefficient(0.0, 0.5, 1e-15);
        assert_eq!(det, 0.0);
    }

    // t=tau のとき det = 1 - 1/e ≈ 0.6321
    #[test]
    fn test_effective_emission_coefficient_at_one_tau() {
        let srh = make_srh(300.0);
        let potential = 0.3;
        let sigma = 1e-15;
        let tau = srh.electron_emission_time(potential, sigma);
        let det = srh.effective_emission_coefficient(tau, potential, sigma);
        let expected = 1.0 - (-1.0_f64).exp();
        assert!(relative_eq!(det, expected, max_relative = 1e-10));
    }

    // t >> tau のとき det は 1 に収束する
    #[test]
    fn test_effective_emission_coefficient_saturates_at_one() {
        let srh = make_srh(300.0);
        let potential = 0.1;
        let sigma = 1e-15;
        let tau = srh.electron_emission_time(potential, sigma);
        let det = srh.effective_emission_coefficient(100.0 * tau, potential, sigma);
        assert!(relative_eq!(det, 1.0, max_relative = 1e-5));
    }

    // 有効質量を重くすると Nc が増えて tau が短くなる
    #[test]
    fn test_set_mass_electron_changes_emission_time() {
        let mut srh = make_srh(300.0);
        let potential = 0.3;
        let sigma = 1e-15;
        let tau_gan = srh.electron_emission_time(potential, sigma);
        srh.set_mass_electron(1.08 * M_ELECTRON); // Silicon の有効質量
        let tau_si = srh.electron_emission_time(potential, sigma);
        // 有効質量大 → Nc 大 → tau 小
        assert!(tau_si < tau_gan);
    }

    // 温度を変えると tau が変化する
    #[test]
    fn test_set_temperature_changes_emission_time() {
        let mut srh = make_srh(300.0);
        let potential = 0.3;
        let sigma = 1e-15;
        let tau_300k = srh.electron_emission_time(potential, sigma);
        srh.set_temperature(400.0);
        let tau_400k = srh.electron_emission_time(potential, sigma);
        // 高温では q_per_kbt が小さくなり exp 項が減少するため tau が短くなる
        assert!(tau_400k < tau_300k);
    }
}
