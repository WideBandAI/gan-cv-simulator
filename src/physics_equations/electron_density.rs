use crate::constants::physics::*;
use crate::physics_equations::band_density::conduction_band_density;

pub trait ElectronDensity {
    fn electron_density(&self, potential: f64, mass_electron: f64, temperature: f64) -> f64;
}

pub struct BoltzmannApproximation {}

impl ElectronDensity for BoltzmannApproximation {
    fn electron_density(&self, potential: f64, mass_electron: f64, temperature: f64) -> f64 {
        let nc = conduction_band_density(mass_electron, temperature);
        let n = nc * (Q_ELECTRON * potential / (K_BOLTZMANN * temperature)).exp();
        n
    }
}
