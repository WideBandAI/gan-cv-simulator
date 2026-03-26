use crate::constants::physics::*;

#[derive(Debug)]
pub struct FermiDiracStatics {
    temperature: f64,
    q_per_kbt: f64,
}

/// Fermi-Dirac statics
///
/// # Arguments
///
/// - `temperature` (`f64`) - temperature in K.
///
/// # Returns
///
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let _ = new();
/// ```
impl FermiDiracStatics {
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

    /// fermi-dirac statics
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
    /// ```
    /// use crate::...;
    ///
    /// let _ = fermi_dirac();
    /// ```
    pub fn fermi_dirac(&self, potential: f64) -> f64 {
        let fermi_dirac = 1.0 / (1.0 + (potential * self.q_per_kbt).exp());
        fermi_dirac
    }
}
