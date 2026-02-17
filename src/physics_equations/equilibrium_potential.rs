use crate::constants::physics::*;
use crate::physics_equations::conduction_band_density::conduction_band_density;

/// Calculate the equilibrium potential for n-type semiconductor.
///
/// # Arguments
///
/// - `effective_mass` (`&f64`) - The effective mass of electron (in m_e).
/// - `donor_concentration` (`&f64`) - The donor concentration (in m^-3).
/// - `temperature` (`&f64`) - The temperature (in K).
///
/// # Returns
///
/// - `f64` - The equilibrium potential (Ec - Ef) (in eV).
pub fn equilibrium_potential_n_type(
    effective_mass: f64,
    donor_concentration: f64,
    temperature: f64,
) -> f64 {
    let nc = conduction_band_density(effective_mass, temperature);
    let phi = (K_BOLTZMANN * temperature / Q_ELECTRON) * (nc / donor_concentration).ln();
    phi
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;
    use test_case::test_case;

    #[test_case(1.08, 1e17, 0.43499 ; "1")]
    #[test_case(1.08, 1e22, 0.13709 ; "2")]
    fn test_equilibrium_potential_n_type(
        effective_mass: f64,
        donor_concentration: f64,
        expected_equilibrium_potential: f64,
    ) {
        let equilibrium_potential =
            equilibrium_potential_n_type(effective_mass, donor_concentration, 300.0);
        assert!(relative_eq!(
            equilibrium_potential,
            expected_equilibrium_potential,
            max_relative = 1e-3
        ));
    }
}
