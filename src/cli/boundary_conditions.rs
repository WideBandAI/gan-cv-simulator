use crate::cli::structure::{DeviceStructure, MaterialType};
use crate::utils::{get_parsed_input, get_parsed_input_with_default};
use std::str::FromStr;

#[derive(Debug)]
pub struct BoundaryConditions {
    pub barrier_height: f64,
    pub ec_ef_bottom: EcEfBottom,
}

#[derive(Debug, Clone)]
pub enum EcEfBottom {
    Value(f64),
    Auto,
}

impl FromStr for EcEfBottom {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "auto" {
            Ok(EcEfBottom::Auto)
        } else {
            match s.parse::<f64>() {
                Ok(value) => Ok(EcEfBottom::Value(value)),
                Err(_) => Err("Invalid input. Please enter a number or 'auto'.".to_string()),
            }
        }
    }
}

pub fn define_boundary_conditions(device_structure: &DeviceStructure) -> BoundaryConditions {
    println!("Define boundary conditions.");

    let barrier_height: f64 = get_parsed_input("Enter the barrier height (in eV): ");

    let ec_ef_bottom: EcEfBottom = if device_structure
        .material_type
        .last()
        .map_or(false, |material_type| {
            *material_type == MaterialType::Semiconductor
        }) {
        get_parsed_input_with_default(
            "Enter the potential difference between the bottom layer's conduction band and fermi level (in eV). default is 'auto': ",
            EcEfBottom::Auto,
        )
    } else {
        let value: f64 = get_parsed_input(
            "Enter the potential difference between the bottom layer's conduction band and fermi level (in eV): ",
        );
        EcEfBottom::Value(value)
    };

    BoundaryConditions {
        barrier_height,
        ec_ef_bottom,
    }
}
