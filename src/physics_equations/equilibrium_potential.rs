use crate::constants::physics::*;

pub fn equilibrium_potential_n_type(
    conduction_band_density: &f64,
    donor_concentration: &f64,
    temperature: &f64,
) -> f64 {
    let phi = (K_BOLTZMANN * temperature / Q_ELECTRON)
        * (conduction_band_density / donor_concentration).ln();
    phi
}
