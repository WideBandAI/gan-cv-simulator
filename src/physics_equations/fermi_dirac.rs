use crate::constants::physics::*;

#[derive(Debug)]
pub struct FermiDiracStatistics {
    temperature: f64,
    q_per_kbt: f64,
}

/// Fermi-Dirac statistics
///
/// # Arguments
///
/// - `temperature` (`f64`) - temperature in K.
///
/// # Returns
///
/// - `FermiDiracStatistics` - A new instance of FermiDiracStatistics.
///
/// # Examples
///
/// ```ignore
/// use crate::physics_equations::fermi_dirac::FermiDiracStatistics;
///
/// let fds = FermiDiracStatistics::new(300.0);
/// let result = fds.fermi_dirac(0.5);
/// ```
impl FermiDiracStatistics {
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

    /// fermi-dirac statistics
    ///
    /// # Arguments
    ///
    /// - `potential` (`f64`) - Ec - Ef in eV.
    ///
    /// # Returns
    ///
    /// - `f64` - electron probability in the conduction band.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::physics_equations::fermi_dirac::FermiDiracStatistics;
    ///
    /// let fds = FermiDiracStatistics::new(300.0);
    /// let result = fds.fermi_dirac(0.5);
    /// ```
    pub fn fermi_dirac(&self, potential: f64) -> f64 {
        1.0 / (1.0 + (potential * self.q_per_kbt).exp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;
    use test_case::test_case;

    #[test_case(0.0, 300.0, 0.5 ; "at_fermi_level_returns_0_5")]
    #[test_case(0.5, 1000.0, 0.003012 ; "above_fermi_level_returns_near_zero")]
    #[test_case(-0.5, 1000.0, 0.99699 ; "below_fermi_level_returns_near_one")]
    fn test_fermi_dirac(potential: f64, temperature: f64, expected: f64) {
        let fds = FermiDiracStatistics::new(temperature);
        let result = fds.fermi_dirac(potential);
        assert!(relative_eq!(result, expected, max_relative = 1e-3));
    }

    #[test]
    fn test_fermi_dirac_symmetry() {
        let fds = FermiDiracStatistics::new(300.0);
        let potential = 0.3;
        assert!(relative_eq!(
            fds.fermi_dirac(potential) + fds.fermi_dirac(-potential),
            1.0,
            max_relative = 1e-10
        ));
    }

    #[test]
    fn test_get_temperature() {
        let fds = FermiDiracStatistics::new(300.0);
        assert_eq!(fds.get_temperature(), 300.0);
    }

    #[test]
    fn test_set_temperature_updates_value() {
        let mut fds = FermiDiracStatistics::new(300.0);
        fds.set_temperature(500.0);
        assert_eq!(fds.get_temperature(), 500.0);
    }

    #[test]
    fn test_set_temperature_changes_fermi_dirac_result() {
        let mut fds = FermiDiracStatistics::new(300.0);
        let result_300 = fds.fermi_dirac(0.5);
        fds.set_temperature(1000.0);
        let result_1000 = fds.fermi_dirac(0.5);
        // Higher temperature → smaller exponent → higher occupation probability above Fermi level
        assert!(result_1000 > result_300);
    }
}
