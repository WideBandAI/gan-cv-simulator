use crate::structure::DeviceStructure;
use crate::utils::{get_parsed_input};

#[derive(Debug)]
pub struct BulkFixedCharge {
    pub layer_id: Vec<u32>,       // Layer ID where the fixed charge is located
    pub charge_density: Vec<f64>, // Fixed charge density in C/m^3
}

#[derive(Debug)]
pub struct InterfaceFixedCharge {
    pub interface_id: Vec<u32>,       // Interface ID between layers
    pub charge_density: Vec<f64>,     // Fixed charge density in C/m^2
}

pub fn define_bulk_fixed_charge(device_structure: &DeviceStructure) -> BulkFixedCharge {
    let mut bulkfixedcharge = BulkFixedCharge {
        layer_id: vec![],
        charge_density: vec![],
    };

    println!("Define bulk fixed charge parameters.");
    let num_layers = device_structure.id.len();

    for i in 0..num_layers {
        println!("Layer {} (Name: {})", i, device_structure.name[i]);
        let charge_density: f64 = get_parsed_input(&format!(
            "Enter fixed charge density (C/cm^3) for layer {}: ",
            device_structure.id[i]
        ));
        bulkfixedcharge.layer_id.push(device_structure.id[i]);
        bulkfixedcharge.charge_density.push(charge_density * 1e6); // Convert from C/cm^3 to C/m^3
    }
    bulkfixedcharge
}

pub fn define_interface_fixed_charge(device_structure: &DeviceStructure) -> InterfaceFixedCharge {
    let mut interfacefixedcharge = InterfaceFixedCharge {
        interface_id: vec![],
        charge_density: vec![],
    };

    println!("Define interface fixed charge parameters.");
    let num_layers = device_structure.id.len();

    for i in 0..(num_layers - 1) {
        println!(
            "Interface {} between Layer {} (Name: {}) and Layer {} (Name: {})",
            i,
            device_structure.id[i],
            device_structure.name[i],
            device_structure.id[i + 1],
            device_structure.name[i + 1]
        );
        let charge_density: f64 = get_parsed_input(&format!(
            "Enter fixed charge density (C/cm^2) for interface {}: ",
            i
        ));
        interfacefixedcharge.interface_id.push(i as u32);
        interfacefixedcharge.charge_density.push(charge_density * 1e4); // Convert from C/cm^2 to C/m^2
    }
    interfacefixedcharge
}
           