use crate::config::structure::DeviceStructure;
use crate::constants::units::PER_CM2_TO_PER_M2;
use crate::physics_equations::interface_states::DIGSModel;
use crate::physics_equations::interface_states::DiscreteModel;
use crate::physics_equations::interface_states::DiscreteStateType;
use crate::utils::{
    get_bool_input, get_input, get_parsed_input_with_default_nonnegative,
    get_parsed_input_with_default_positiveint,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ContinuousInterfaceStatesConfig {
    pub interface_id: Vec<u32>,     // Interface ID between layers
    pub parameters: Vec<DIGSModel>, // DIGS model parameters
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DiscreteInterfaceStatesConfig {
    pub interface_id: Vec<u32>,              // Interface ID between layers
    pub parameters: Vec<Vec<DiscreteModel>>, // Discrete model parameters
}

pub fn define_interface_states(
    device_structure: &DeviceStructure,
) -> (
    ContinuousInterfaceStatesConfig,
    DiscreteInterfaceStatesConfig,
) {
    let mut continuous_interface_states_config = ContinuousInterfaceStatesConfig {
        interface_id: vec![],
        parameters: vec![],
    };
    let mut discrete_interface_states_config = DiscreteInterfaceStatesConfig {
        interface_id: vec![],
        parameters: vec![],
    };

    let num_layers = device_structure.id.len();
    for i in 0..(num_layers - 1) {
        let has_continuous_traps: bool = get_bool_input(&format!(
            "Does interface between {} and {} have continuous traps? (y/n). Default is n: ",
            device_structure.name[i],
            device_structure.name[i + 1]
        ));
        if has_continuous_traps {
            let params = configure_continuous_interface_states(i, device_structure);
            continuous_interface_states_config
                .interface_id
                .push(i as u32);
            continuous_interface_states_config.parameters.push(params);
        }

        let has_discrete_traps: bool = get_bool_input(&format!(
            "Does interface between {} and {} have discrete traps? (y/n). Default is n: ",
            device_structure.name[i],
            device_structure.name[i + 1]
        ));
        if has_discrete_traps {
            let params = configure_discrete_interface_states(i, device_structure);
            discrete_interface_states_config.interface_id.push(i as u32);
            discrete_interface_states_config.parameters.push(params);
        }
    }
    (
        continuous_interface_states_config,
        discrete_interface_states_config,
    )
}

fn get_discrete_state_type() -> DiscreteStateType {
    loop {
        let input = get_input("Enter state type DonorLike (d) or AcceptorLike (a): ");
        match input.trim().to_lowercase().as_str() {
            "d" => return DiscreteStateType::DonorLike,
            "a" => return DiscreteStateType::AcceptorLike,
            _ => println!("Invalid input. Please enter 'd' or 'a'."),
        }
    }
}

fn configure_continuous_interface_states(
    i: usize,
    device_structure: &DeviceStructure,
) -> DIGSModel {
    let bandgap: f64 =
        device_structure.bandgap_energy[i].min(device_structure.bandgap_energy[i + 1]);

    let dit0: f64 = get_parsed_input_with_default_nonnegative(
        &format!("Enter Dit0 (cm^-2) for interface {}: default is 1e12 ", i),
        1e12,
    );
    let dit0 = dit0 * PER_CM2_TO_PER_M2; // Convert to m^-2 for internal use
    let nssec: f64 = get_parsed_input_with_default_nonnegative(
        &format!("Enter nssec for interface {}: default is 10 ", i),
        10.0,
    );
    let nssev: f64 = get_parsed_input_with_default_nonnegative(
        &format!("Enter nssev for interface {}: default is 10 ", i),
        10.0,
    );
    let ecnl: f64 = get_parsed_input_with_default_nonnegative(
        &format!(
            "Enter |Ec - Ecnl| (eV) for interface {}: default is 1.3 ",
            i
        ),
        1.3,
    );
    let nd: f64 = get_parsed_input_with_default_nonnegative(
        &format!("Enter nd for interface {}: default is 3 ", i),
        3.0,
    );
    let na: f64 = get_parsed_input_with_default_nonnegative(
        &format!("Enter na for interface {}: default is 3 ", i),
        3.0,
    );
    DIGSModel::new(dit0, nssec, nssev, ecnl, nd, na, bandgap)
}

fn configure_discrete_interface_states(
    i: usize,
    device_structure: &DeviceStructure,
) -> Vec<DiscreteModel> {
    let bandgap: f64 =
        device_structure.bandgap_energy[i].min(device_structure.bandgap_energy[i + 1]);
    let num_discrete_traps: u32 = get_parsed_input_with_default_positiveint(
        &format!(
            "Enter the number of discrete traps for interface {}: default is 1 ",
            i
        ),
        1,
    );
    let mut discrete_parameters = Vec::new();
    for j in 0..num_discrete_traps {
        let ditmax: f64 = get_parsed_input_with_default_nonnegative(
            &format!(
                "Enter Ditmax (cm^-2) for interface {} discrete trap {}: default is 1e12 ",
                i, j
            ),
            1e12,
        );
        let ditmax = ditmax * PER_CM2_TO_PER_M2; // Convert to m^-2 for internal use
        let ed: f64 = get_parsed_input_with_default_nonnegative(
            &format!(
                "Enter |Ec - Ed| (eV) for interface {} discrete trap {}: default is 0.5 ",
                i, j
            ),
            0.5,
        );
        let fwhm: f64 = get_parsed_input_with_default_nonnegative(
            &format!(
                "Enter FWHM (eV) for interface {} discrete trap {}: default is 0.3 ",
                i, j
            ),
            0.3,
        );
        let state_type: DiscreteStateType = get_discrete_state_type();
        discrete_parameters.push(DiscreteModel::new(ditmax, ed, fwhm, state_type, bandgap));
    }
    discrete_parameters
}
