use crate::constants::physics::*;

/// Ionized donor density
///
/// # Arguments
///
/// - `donor_density` (`f64`) - The total donor density in the material.
/// - `temperature` (`f64`) - The temperature of the material in Kelvin.
/// - `potential` (`f64`) - Ed - Ef in eV. Ed is the donor energy level and Ef is the Fermi level.
///
/// # Returns
///
/// - `f64` - The ionized donor density in the material.
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let _ = ionized_donor_density();
/// ```
pub fn ionized_donor_density(donor_density: f64, temperature: f64, potential: f64) -> f64 {
    let ion_nd =
        donor_density / (1.0 + 2.0 * (-potential * Q_ELECTRON / (K_BOLTZMANN * temperature)).exp());
    ion_nd
}
