use crate::config::measurement::Measurement;
use crate::config::structure::{DeviceStructure, MaterialType};
use crate::physics_equations::fermi_dirac_density::CarrierStatistics;
use crate::utils::{get_parsed_input, get_parsed_input_with_default};

/// Boundary conditions for the potential solver.
///
/// # Fields
///
/// - `barrier_height` (`f64`) - The barrier height in eV.
/// - `ec_ef_bottom` (`f64`) - The potential difference between the bottom layer's conduction band and fermi level in eV.
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let s = BoundaryConditions {
///     barrier_height: value,
///     ec_ef_bottom: value,
/// };
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BoundaryConditions {
    pub barrier_height: f64,
    pub ec_ef_bottom: f64,
}

/// Define boundary conditions.
///
/// # Arguments
///
/// - `device_structure` (`&DeviceStructure`) - The device structure to base boundary conditions on.
/// - `measurement` (`&Measurement`) - The measurement parameters to base boundary conditions on.
///
/// # Returns
///
/// - `BoundaryConditions` - The boundary conditions derived from the device structure and measurement parameters.
pub fn define_boundary_conditions(
    device_structure: &DeviceStructure,
    measurement: &Measurement,
) -> BoundaryConditions {
    define_boundary_conditions_with_statistics(
        device_structure,
        measurement,
        CarrierStatistics::Boltzmann,
    )
}

/// Define boundary conditions using the selected fully ionized bulk statistics.
pub fn define_boundary_conditions_with_statistics(
    device_structure: &DeviceStructure,
    measurement: &Measurement,
    statistics: CarrierStatistics,
) -> BoundaryConditions {
    println!("Define boundary conditions.");

    let barrier_height: f64 = get_parsed_input("Enter the barrier height (in eV): ");

    let ec_ef_bottom: f64 = if let (Some(&MaterialType::Semiconductor), Some(me), Some(nd)) = (
        device_structure.material_type.last(),
        device_structure.mass_electron.last(),
        device_structure.donor_concentration.last(),
    ) {
        let equilibrium_potential =
            statistics.equilibrium_potential(*me, *nd, measurement.temperature.temperature);
        get_parsed_input_with_default(
            "Enter the potential difference between the bottom layer's conduction band and fermi level (in eV). default is equilibrium potential: ",
            equilibrium_potential,
        )
    } else {
        get_parsed_input(
            "Enter the potential difference between the bottom layer's conduction band and fermi level (in eV): ",
        )
    };

    BoundaryConditions {
        barrier_height,
        ec_ef_bottom,
    }
}
