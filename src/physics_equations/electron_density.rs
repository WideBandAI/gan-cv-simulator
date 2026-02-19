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
