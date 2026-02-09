use crate::constants::physics::*;

/// Calculate the equilibrium potential for n-type semiconductor.
///
/// # Arguments
///
/// - `conduction_band_density` (`&f64`) - The effective density of states in the conduction band (in m^-3).
/// - `donor_concentration` (`&f64`) - The donor concentration (in m^-3).
/// - `temperature` (`&f64`) - The temperature (in K).
///
/// # Returns
///
/// - `f64` - The equilibrium potential (Ec - Ef) (in eV).
pub fn equilibrium_potential_n_type(
    conduction_band_density: &f64,
    donor_concentration: &f64,
    temperature: &f64,
) -> f64 {
    let phi = (K_BOLTZMANN * temperature / Q_ELECTRON)
        * (conduction_band_density / donor_concentration).ln();
    phi
}
