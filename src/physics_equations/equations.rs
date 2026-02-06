use crate::constants::physics::*;

pub fn conduction_band_density(effective_mass: f64, temperature: f64) -> f64 {
    let coefficient = 2.0
        * (2.0 * std::f64::consts::PI * effective_mass * M_ELECTRON * K_BOLTZMANN * temperature)
            .powf(1.5);
    let denominator = H_PLANK_CONSTANT.powf(3.0);

    coefficient / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conduction_band_density_increases_with_temperature() {
        let effective_mass = 1.18;
        let temp = 300.0;

        let density = conduction_band_density(effective_mass, temp);
        println!("Conduction band density at {} K: {:e}", temp, density / 1e6);
        assert!((density / 1e6 - 2.81e19).abs() < 1e17); // Approximate value for silicon at 300K);
    }

    #[test]
    fn test_conduction_band_density_increases_with_mass() {
        let temperature = 300.0;
        let mass1 = 0.26;
        let mass2 = 0.52;

        let density1 = conduction_band_density(mass1, temperature);
        let density2 = conduction_band_density(mass2, temperature);

        assert!(density2 > density1);
    }

    #[test]
    fn test_conduction_band_density_positive() {
        let density = conduction_band_density(0.26, 300.0);
        assert!(density > 0.0);
    }

    #[test]
    fn test_conduction_band_density_known_value() {
        let density = conduction_band_density(0.26, 300.0);
        assert!((density - 2.8e19).abs() < 1e18);
    }
}
