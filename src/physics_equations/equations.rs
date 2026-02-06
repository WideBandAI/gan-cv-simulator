use crate::constants::physics::*;

pub fn conduction_band_density(effective_mass: f64, temperature: f64) -> f64 {
    let coefficient =
        2.0 * (2.0 * std::f64::consts::PI * effective_mass * K_BOLTZMANN * temperature).powf(1.5);
    let denominator = H_PLANK_CONSTANT.powf(3.0);

    coefficient / denominator
}
