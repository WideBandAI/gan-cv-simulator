use crate::config::interface_states::{
    ContinuousInterfaceStatesConfig, DiscreteInterfaceStatesConfig,
};
use crate::config::structure::DeviceStructure;
use crate::constants::physics::M_ELECTRON;
use crate::constants::units::CM2_TO_M2;
use crate::utils::{
    get_input, get_parsed_input, get_parsed_input_with_default,
    get_parsed_input_with_default_nonnegative,
};
use itertools::Itertools;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum CaptureCrossSectionModel {
    /// Constant model: σ = σ₀ [m²] (stored in SI units)
    Constant { sigma: f64 },
    /// Energy-dependent model: σ(E) = σ_mid * exp((E - E_mid) / E_slope) [m²] (stored in SI units)
    /// Positive e_slope: cross-section increases for E > E_mid.
    EnergyDependent {
        sigma_mid: f64,
        e_mid: f64,
        e_slope: f64,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CaptureCrossSectionConfig {
    pub interface_id: Vec<u32>,
    pub model: Vec<CaptureCrossSectionModel>,
    pub mass_electron: Vec<f64>,
}

/// Collect the sorted, deduplicated union of interface IDs that have any interface states.
fn collect_interface_ids(
    continuous: &ContinuousInterfaceStatesConfig,
    discrete: &DiscreteInterfaceStatesConfig,
) -> Vec<u32> {
    continuous
        .interface_id
        .iter()
        .chain(discrete.interface_id.iter())
        .copied()
        .collect::<std::collections::BTreeSet<u32>>()
        .into_iter()
        .collect()
}

pub fn define_capture_cross_section(
    continuous: &ContinuousInterfaceStatesConfig,
    discrete: &DiscreteInterfaceStatesConfig,
    device_structure: &DeviceStructure,
) -> CaptureCrossSectionConfig {
    let interface_ids = collect_interface_ids(continuous, discrete);

    if !interface_ids.is_empty() {
        println!("Define capture cross-section parameters.");

        let (interface_id, model, mass_electron): (Vec<_>, Vec<_>, Vec<_>) = interface_ids
            .iter()
            .map(|&id| {
                println!(
                    "Interface between Layer {} and Layer {}:",
                    device_structure.name[id as usize],
                    device_structure.name[id as usize + 1]
                );
                let model = get_capture_cross_section_model(id, device_structure);
                // Interface id is a boundary index (0..num_layers-1), so id+1 is always a valid layer index.
                let lower_layer_mass = *device_structure
                    .mass_electron
                    .get(id as usize + 1)
                    .expect("Interface id must be less than the number of layers minus one");
                let me = get_mass_electron(id, lower_layer_mass, device_structure);
                (id, model, me)
            })
            .multiunzip();

        CaptureCrossSectionConfig {
            interface_id,
            model,
            mass_electron,
        }
    } else {
        println!("No interfaces with states defined, skipping capture cross-section parameters.");
        CaptureCrossSectionConfig {
            interface_id: vec![],
            model: vec![],
            mass_electron: vec![],
        }
    }
}

fn get_mass_electron(
    interface_id: u32,
    lower_layer_mass_kg: f64,
    device_structure: &DeviceStructure,
) -> f64 {
    loop {
        let coeff: f64 = if lower_layer_mass_kg > 0.0 {
            let lower_mass_coeff = lower_layer_mass_kg / M_ELECTRON;
            get_parsed_input_with_default(
                &format!(
                    "Enter effective mass coefficient of electron for interface between {} and {}: default is {:.4} ",
                    device_structure.name[interface_id as usize],
                    device_structure.name[interface_id as usize + 1],
                    lower_mass_coeff
                ),
                lower_mass_coeff,
            )
        } else {
            get_parsed_input(&format!(
                "Enter effective mass coefficient of electron for interface between {} and {}: ",
                device_structure.name[interface_id as usize],
                device_structure.name[interface_id as usize + 1]
            ))
        };

        if coeff <= 0.0 {
            println!("Invalid input. Please enter a positive value.");
        } else {
            return coeff * M_ELECTRON;
        }
    }
}

fn get_capture_cross_section_model(
    interface_id: u32,
    device_structure: &DeviceStructure,
) -> CaptureCrossSectionModel {
    loop {
        let input = get_input(&format!(
            "Select capture cross-section model for interface between {} and {}: Constant (c) or Energy-dependent (e): default is c ",
            device_structure.name[interface_id as usize],
            device_structure.name[interface_id as usize + 1]
        ));
        match input.trim().to_lowercase().as_str() {
            "c" | "" => {
                let sigma_cm2: f64 = get_parsed_input_with_default_nonnegative(
                    &format!(
                        "Enter sigma (cm^2) for interface between {} and {}: default is 1e-16 ",
                        device_structure.name[interface_id as usize],
                        device_structure.name[interface_id as usize + 1]
                    ),
                    1e-16,
                );
                return CaptureCrossSectionModel::Constant {
                    sigma: sigma_cm2 * CM2_TO_M2,
                };
            }
            "e" => {
                let sigma_mid_cm2: f64 = get_parsed_input_with_default_nonnegative(
                    &format!(
                        "Enter sigma_mid (cm^2) for interface between {} and {}: default is 1e-16 ",
                        device_structure.name[interface_id as usize],
                        device_structure.name[interface_id as usize + 1]
                    ),
                    1e-16,
                );
                let e_mid: f64 = get_parsed_input_with_default_nonnegative(
                    &format!(
                        "Enter E_mid (eV) for interface between {} and {}: default is 0.5 ",
                        device_structure.name[interface_id as usize],
                        device_structure.name[interface_id as usize + 1]
                    ),
                    0.5,
                );
                let e_slope: f64 = get_parsed_input_with_default(
                    &format!(
                        "Enter E_slope (eV) for interface between {} and {}: default is 0.1 ",
                        device_structure.name[interface_id as usize],
                        device_structure.name[interface_id as usize + 1]
                    ),
                    0.1,
                );
                return CaptureCrossSectionModel::EnergyDependent {
                    sigma_mid: sigma_mid_cm2 * CM2_TO_M2,
                    e_mid,
                    e_slope,
                };
            }
            _ => println!("Invalid input. Please enter 'c' or 'e'."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics_equations::interface_states::{DIGSModel, DiscreteModel, DiscreteStateType};

    fn make_continuous(ids: Vec<u32>) -> ContinuousInterfaceStatesConfig {
        let n = ids.len();
        ContinuousInterfaceStatesConfig {
            interface_id: ids,
            parameters: vec![DIGSModel::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0); n],
        }
    }

    fn make_discrete(ids: Vec<u32>) -> DiscreteInterfaceStatesConfig {
        let n = ids.len();
        DiscreteInterfaceStatesConfig {
            interface_id: ids,
            parameters: vec![
                vec![DiscreteModel::new(
                    1.0,
                    0.5,
                    0.1,
                    DiscreteStateType::DonorLike,
                    1.0
                )];
                n
            ],
        }
    }

    #[test]
    fn test_constant_model_stores_sigma_in_si_units() {
        let sigma_cm2 = 1e-16_f64;
        let sigma_m2 = sigma_cm2 * CM2_TO_M2;
        let model = CaptureCrossSectionModel::Constant { sigma: sigma_m2 };
        match model {
            CaptureCrossSectionModel::Constant { sigma } => {
                assert_eq!(sigma, sigma_m2);
            }
            _ => panic!("Expected Constant"),
        }
    }

    #[test]
    fn test_energy_dependent_model_stores_sigma_mid_in_si_units() {
        let sigma_mid_cm2 = 1e-16_f64;
        let sigma_mid_m2 = sigma_mid_cm2 * CM2_TO_M2;
        let model = CaptureCrossSectionModel::EnergyDependent {
            sigma_mid: sigma_mid_m2,
            e_mid: 0.5,
            e_slope: 0.1,
        };
        match model {
            CaptureCrossSectionModel::EnergyDependent {
                sigma_mid,
                e_mid,
                e_slope,
            } => {
                assert_eq!(sigma_mid, sigma_mid_m2);
                // No arithmetic is applied to e_mid/e_slope; stored bits are identical to input literals.
                assert_eq!(e_mid, 0.5);
                assert_eq!(e_slope, 0.1);
            }
            _ => panic!("Expected EnergyDependent"),
        }
    }

    #[test]
    fn test_collect_interface_ids_union_of_continuous_and_discrete() {
        let continuous = make_continuous(vec![0, 2]);
        let discrete = make_discrete(vec![1, 2]);
        let ids = collect_interface_ids(&continuous, &discrete);
        assert_eq!(ids, vec![0, 1, 2]);
    }

    #[test]
    fn test_collect_interface_ids_deduplicates() {
        let continuous = make_continuous(vec![0, 1]);
        let discrete = make_discrete(vec![0, 1]);
        let ids = collect_interface_ids(&continuous, &discrete);
        assert_eq!(ids, vec![0, 1]);
    }

    #[test]
    fn test_collect_interface_ids_empty_discrete() {
        let continuous = make_continuous(vec![0]);
        let discrete = DiscreteInterfaceStatesConfig {
            interface_id: vec![],
            parameters: vec![],
        };
        let ids = collect_interface_ids(&continuous, &discrete);
        assert_eq!(ids, vec![0]);
    }

    #[test]
    fn test_collect_interface_ids_empty_continuous() {
        let continuous = ContinuousInterfaceStatesConfig {
            interface_id: vec![],
            parameters: vec![],
        };
        let discrete = make_discrete(vec![1]);
        let ids = collect_interface_ids(&continuous, &discrete);
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn test_collect_interface_ids_both_empty() {
        let continuous = ContinuousInterfaceStatesConfig {
            interface_id: vec![],
            parameters: vec![],
        };
        let discrete = DiscreteInterfaceStatesConfig {
            interface_id: vec![],
            parameters: vec![],
        };
        let ids = collect_interface_ids(&continuous, &discrete);
        assert!(ids.is_empty());
    }
}
