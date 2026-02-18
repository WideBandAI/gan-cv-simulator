use crate::constants::physics::Q_ELECTRON;

pub trait ElectronDensity {
    fn electron_density(&self, potential: f64) -> f64;
}

pub struct BoltzmannApproximation {
    pub mass_electron: f64,
    pub donor_concentration: f64,
    pub temperature: f64,
}

impl ElectronDensity for BoltzmannApproximation {
    fn electron_density(&self, potential: f64) -> f64 {
        let nc = conduction_band_density(self.mass_electron, self.temperature);
        let phi = (K_BOLTZMANN * temperature / Q_ELECTRON) * (nc / donor_concentration).ln();
        phi
    }
}
