use crate::constants::physics::*;

pub fn conduction_band_density(effective_mass_coefficient: f64, temperature: f64) -> f64 {
    let coefficient = 2.0
        * (2.0
            * std::f64::consts::PI
            * effective_mass_coefficient
            * M_ELECTRON
            * K_BOLTZMANN
            * temperature)
            .powf(1.5);
    let denominator = H_PLANK_CONSTANT.powf(3.0);

    coefficient / denominator
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;

    #[test]
    fn test_conduction_band_density() {
        let effective_mass_coefficient = 1.08;
        let temp = 300.0;

        let density = conduction_band_density(effective_mass_coefficient, temp);
        let _ = relative_eq!(density, 2.81e25, max_relative = 1e-3);
    }
}
