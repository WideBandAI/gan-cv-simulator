use crate::config::structure::DeviceStructure;
use crate::physics_equations::interface_states::DIGSModel;
use crate::physics_equations::interface_states::DiscreteModel;
use crate::utils::get_parsed_input_with_default;

#[derive(Debug)]
pub struct ContinuousInterfaceStatesConfig {
    pub interface_id: Vec<u32>,     // Interface ID between layers
    pub parameters: Vec<DIGSModel>, // DIGS model parameters
}

#[derive(Debug)]
pub struct DiscreteInterfaceStatesConfig {
    pub interface_id: Vec<u32>,         // Interface ID between layers
    pub parameters: Vec<DiscreteModel>, // Discrete model parameters
}

pub fn define_interface_states(device_structure: &DeviceStructure) {
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
        let has_continuous_traps: bool = get_parsed_input_with_default(
            &format!(
                "Does interface {} have continuous traps? (true/false): default is false ",
                i
            ),
            false,
        );
        if has_continuous_traps {
            let bandgap: f64 =
                device_structure.bandgap_energy[i].min(device_structure.bandgap_energy[i + 1]);

            let dit0: f64 = get_parsed_input_with_default(
                &format!("Enter Dit0 (cm^-2) for interface {}: default is 1e12 ", i),
                1e12,
            );
            let nssec: f64 = get_parsed_input_with_default(
                &format!("Enter nssec for interface {}: default is 10 ", i),
                10.0,
            );
            let nssev: f64 = get_parsed_input_with_default(
                &format!("Enter nssev for interface {}: default is 10 ", i),
                10.0,
            );
            let ecnl: f64 = loop {
                let val: f64 = get_parsed_input_with_default(
                    &format!(
                        "Enter |Ec - Ecnl| (eV) for interface {}: default is 1.3 ",
                        i
                    ),
                    1.3,
                );
                if val < 0.0 || val > bandgap {
                    println!(
                        "Error: |Ec - Ecnl| must be between 0 and {:.3}. Please enter a valid value.",
                        bandgap
                    );
                } else {
                    break val;
                }
            };
            let nd: f64 = get_parsed_input_with_default(
                &format!("Enter nd for interface {}: default is 3 ", i),
                3.0,
            );
            let na: f64 = get_parsed_input_with_default(
                &format!("Enter na for interface {}: default is 3 ", i),
                3.0,
            );
        }
    }
}
