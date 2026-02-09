use crate::cli::measurement::Measurement;
use crate::cli::structure::{DeviceStructure, MaterialType};
use crate::physics_equations::equilibrium_potential::equilibrium_potential_n_type;
use crate::utils::{get_parsed_input, get_parsed_input_with_default};

#[derive(Debug)]
pub struct BoundaryConditions {
    pub barrier_height: f64,
    pub ec_ef_bottom: f64,
}

pub fn define_boundary_conditions(
    device_structure: &DeviceStructure,
    measurement: &Measurement,
) -> BoundaryConditions {
    println!("Define boundary conditions.");

    let barrier_height: f64 = get_parsed_input("Enter the barrier height (in eV): ");

    let ec_ef_bottom: f64 = if device_structure
        .material_type
        .last()
        .map_or(false, |material_type| {
            *material_type == MaterialType::Semiconductor
        }) {
        let equilibrium_potential = equilibrium_potential_n_type(
            device_structure.nc.last().unwrap(),
            device_structure.nd.last().unwrap(),
            &measurement.temperature.temperature,
        );
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
