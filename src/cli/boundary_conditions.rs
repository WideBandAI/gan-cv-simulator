use crate::cli::structure::{DeviceStructure, MaterialType};
use crate::utils::get_parsed_input;

#[derive(Debug)]
pub struct BoundaryConditions {
    pub barrier_height: f64,
    pub ec_ef_bottom: f64,
}

pub fn define_boundary_conditions(device_structure: &DeviceStructure) -> BoundaryConditions {
    println!("Define boundary conditions.");

    let barrier_height: f64 = get_parsed_input("Enter the barrier height (in eV): ");

    if device_structure
        .material_type
        .last()
        .map_or(false, |material_type| {
            *material_type == MaterialType::Semiconductor
        })
    {
        println!("The last layer is a semiconductor.");
    }

    let ec_ef_bottom: f64 = get_parsed_input(
        "Enter the potential difference between the bottom layer's conduction band and fermi level (in eV): ",
    );

    BoundaryConditions {
        barrier_height,
        ec_ef_bottom,
    }
}
