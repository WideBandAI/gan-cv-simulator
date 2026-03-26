use crate::constants::physics::*;
use crate::physics_equations::band_density::ConductionBandDensity;

#[derive(Debug)]
pub struct SRHStatics {
    temperature: f64,
    q_per_kbt: f64,
    thermal_velocity: f64,
    conduction_band_density: f64,
}

impl SRHStatics {
    pub fn new(temperature: f64, mass_electron: f64, thermal_velocity: f64) -> Self {
        let conduction_band_density =
            ConductionBandDensity::new(temperature).conduction_band_density(mass_electron);
        Self {
            temperature,
            q_per_kbt: Q_ELECTRON / (K_BOLTZMANN * temperature),
            thermal_velocity,
            conduction_band_density,
        }
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
        self.q_per_kbt = Q_ELECTRON / (K_BOLTZMANN * temperature);
        self.conduction_band_density =
            ConductionBandDensity::new(temperature).conduction_band_density(M_ELECTRON);
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

    pub fn electron_emission_time(&self, potential: f64, capture_cross_section: f64) -> f64 {
        let tau = (potential * self.q_per_kbt).exp()
            / (self.thermal_velocity * capture_cross_section * self.conduction_band_density);
        tau
    }

    pub fn effective_emission_coefficient(
        &self,
        time: f64,
        potential: f64,
        capture_cross_section: f64,
    ) -> f64 {
        let tau = self.electron_emission_time(potential, capture_cross_section);
        let det = 1.0 - (-time / tau).exp();
        det
    }
}
