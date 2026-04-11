use crate::constants::physics::{EPSILON_0, M_ELECTRON};
use crate::constants::units::{NM_TO_M, PER_CM3_TO_PER_M3};
use crate::utils::{get_input, get_parsed_input};
use std::vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MaterialType {
    Semiconductor,
    Insulator,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeviceStructure {
    pub id: Vec<u32>,      // Optional: layer ID
    pub name: Vec<String>, // Optional: name of the device structure
    pub material_type: Vec<MaterialType>,
    pub thickness: Vec<f64>,             // meters
    pub mass_electron: Vec<f64>,         // effective mass of electron
    pub permittivity: Vec<f64>,          // absolute permittivity in F/m
    pub bandgap_energy: Vec<f64>,        // bandgap energy in eV
    pub delta_conduction_band: Vec<f64>, // delta conduction band in eV from bottom layer to current layer
    pub donor_concentration: Vec<f64>,   // donor concentration in m^-3
    pub energy_level_donor: Vec<f64>,    // energy level of donor in eV (Ec-Ed)
}

fn get_material_type(prompt: &str) -> MaterialType {
    loop {
        let input = get_input(prompt);
        match input.trim().to_lowercase().as_str() {
            "s" => return MaterialType::Semiconductor,
            "i" => return MaterialType::Insulator,
            _ => println!("Invalid input. Please enter 's' or 'i'."),
        }
    }
}

/// Define the device structure by prompting the user for input.
///
/// # Returns
///
/// - `DeviceStructure` - The defined device structure.
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let _ = define_structure();
/// ```
pub fn define_structure() -> DeviceStructure {
    println!("Define the structure.");
    let num_layers: u32 = get_parsed_input("Enter the number of layers: ");
    println!("Number of layers: {}", num_layers);

    let mut device = DeviceStructure {
        id: vec![],
        name: vec![],
        material_type: vec![],
        thickness: vec![],
        mass_electron: vec![],
        permittivity: vec![],
        bandgap_energy: vec![],
        delta_conduction_band: vec![],
        donor_concentration: vec![],
        energy_level_donor: vec![],
    };

    for n in 0..(num_layers) {
        device.id.push(n);
        println!("\nLayer {}:", n);

        let name = get_input(&format!(
            "Enter name for layer {} (or press Enter to skip): ",
            n
        ));
        device.name.push(name.trim().to_string());

        let mat_type = get_material_type(&format!(
            "Is layer {} a Semiconductor (s) or Insulator (i)? ",
            n
        ));
        device.material_type.push(mat_type);

        let thickness_nm: f64 = get_parsed_input(&format!("Enter thickness of layer {} (nm): ", n));
        device.thickness.push(thickness_nm * NM_TO_M); // convert nm to meters

        let permittivity: f64 = get_parsed_input(&format!(
            "Enter relative permittivity coefficient for layer {}: ",
            n
        ));
        device.permittivity.push(permittivity * EPSILON_0); // convert relative permittivity to absolute

        let eg: f64 = get_parsed_input(&format!("Enter bandgap energy in eV for layer {}: ", n));
        device.bandgap_energy.push(eg);

        if n == (num_layers - 1) {
            device.delta_conduction_band.push(0.0); // last layer delta conduction band is 0
        } else {
            let dec: f64 = get_parsed_input(&format!(
                "Enter delta conduction band in eV from bottom layer to layer {}: ",
                n
            ));
            device.delta_conduction_band.push(dec);
        }

        if device.material_type[n as usize] == MaterialType::Semiconductor {
            let me: f64 = get_parsed_input(&format!(
                "Enter effective mass coefficient of electron for layer {}: ",
                n
            ));
            device.mass_electron.push(me * M_ELECTRON); // convert to units of electron mass

            let nd: f64 = get_parsed_input(&format!(
                "Enter donor concentration in cm^-3 for layer {}: ",
                n
            ));
            device.donor_concentration.push(nd * PER_CM3_TO_PER_M3); // convert cm^-3 to m^-3

            let end: f64 = get_parsed_input(&format!(
                "Enter energy level of donor in eV (Ec-Ed) for layer {}: ",
                n
            ));
            device.energy_level_donor.push(end);
        } else {
            device.mass_electron.push(0.0);
            device.donor_concentration.push(0.0);
            device.energy_level_donor.push(0.0);
        }
    }
    device
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_type_equality() {
        assert_eq!(MaterialType::Semiconductor, MaterialType::Semiconductor);
        assert_eq!(MaterialType::Insulator, MaterialType::Insulator);
        assert_ne!(MaterialType::Semiconductor, MaterialType::Insulator);
    }

    #[test]
    fn test_device_structure_creation() {
        let device = DeviceStructure {
            id: vec![0],
            name: vec!["test".to_string()],
            material_type: vec![MaterialType::Semiconductor],
            thickness: vec![1e-8],
            mass_electron: vec![0.5],
            permittivity: vec![12.0],
            bandgap_energy: vec![1.12],
            delta_conduction_band: vec![0.0],
            donor_concentration: vec![1e16],
            energy_level_donor: vec![0.1],
        };

        assert_eq!(device.material_type.len(), 1);
        assert_eq!(device.thickness[0], 1e-8);
        assert_eq!(device.mass_electron[0], 0.5);
        assert_eq!(device.permittivity[0], 12.0);
    }

    #[test]
    fn test_device_structure_multiple_layers() {
        let device = DeviceStructure {
            id: vec![0, 1],
            name: vec!["layer1".to_string(), "layer2".to_string()],
            material_type: vec![MaterialType::Semiconductor, MaterialType::Insulator],
            thickness: vec![1e-8, 2e-8],
            mass_electron: vec![0.5, 0.0],
            permittivity: vec![12.0, 3.9],
            bandgap_energy: vec![1.12, 9.0],
            delta_conduction_band: vec![0.3, 0.0],
            donor_concentration: vec![1e16, 0.0],
            energy_level_donor: vec![0.1, 0.0],
        };

        assert_eq!(device.material_type.len(), 2);
        assert_eq!(device.thickness.len(), 2);
    }
}
