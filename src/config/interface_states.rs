use crate::config::structure::DeviceStructure;
use crate::physics_equations::interface_states::DIGSModel;
use crate::physics_equations::interface_states::DiscreteModel;
use crate::physics_equations::interface_states::DiscreteStateType;
use crate::utils::{
    get_parsed_input_with_default, get_parsed_input_with_default_nonnegative,
    get_parsed_input_with_default_positiveint,
};

#[derive(Debug)]
pub struct ContinuousInterfaceStatesConfig {
    pub interface_id: Vec<u32>,     // Interface ID between layers
    pub parameters: Vec<DIGSModel>, // DIGS model parameters
}

#[derive(Debug)]
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

            let dit0: f64 = get_parsed_input_with_default_nonnegative(
                &format!("Enter Dit0 (cm^-2) for interface {}: default is 1e12 ", i),
                1e12,
            );
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
            continuous_interface_states_config
                .interface_id
                .push(i as u32);
            continuous_interface_states_config
                .parameters
                .push(DIGSModel::new(dit0, nssec, nssev, ecnl, nd, na, bandgap));
        }

        let has_discrete_traps: bool = get_parsed_input_with_default(
            &format!(
                "Does interface {} have discrete traps? (true/false): default is false ",
                i
            ),
            false,
        );
        if has_discrete_traps {
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
                let ed: f64 = get_parsed_input_with_default_nonnegative(
                    &format!(
                        "Enter |Ec - Ed| (eV) for interface {} discrete trap {}: default is 0.6 ",
                        i, j
                    ),
                    0.6,
                );
                let fwhm: f64 = get_parsed_input_with_default_nonnegative(
                    &format!(
                        "Enter FWHM (eV) for interface {} discrete trap {}: default is 0.1 ",
                        i, j
                    ),
                    0.1,
                );
                let state_type: DiscreteStateType = get_parsed_input_with_default(
                    &format!(
                        "Enter state type for interface {} discrete trap {}: default is DonorLike ",
                        i, j
                    ),
                    DiscreteStateType::DonorLike,
                );
                discrete_parameters.push(DiscreteModel::new(ditmax, ed, fwhm, state_type));
            }
            discrete_interface_states_config.interface_id.push(i as u32);
            discrete_interface_states_config
                .parameters
                .push(discrete_parameters);
        }
    }
    (
        continuous_interface_states_config,
        discrete_interface_states_config,
    )
}
