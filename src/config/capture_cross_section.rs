use crate::config::interface_states::{ContinuousInterfaceStatesConfig, DiscreteInterfaceStatesConfig};
use crate::constants::units::CM_TO_M;
use crate::utils::{get_input, get_parsed_input_with_default, get_parsed_input_with_default_nonnegative};

#[derive(Debug, Clone, Copy)]
pub enum CaptureCrossSectionModel {
    /// Constant model: σ = σ₀ [m²] (stored in SI units)
    Constant { sigma: f64 },
    /// Energy-dependent model: σ(E) = σ_mid * exp((E - E_mid) / E_slope) [m²] (stored in SI units)
    /// Positive e_slope: cross-section increases for E > E_mid.
    EnergyDependent { sigma_mid: f64, e_mid: f64, e_slope: f64 },
}

#[derive(Debug)]
pub struct CaptureCrossSectionConfig {
    pub interface_id: Vec<u32>,
    pub model: Vec<CaptureCrossSectionModel>,
}

/// Collect the sorted, deduplicated union of interface IDs that have any interface states.
fn collect_interface_ids(
    continuous: &ContinuousInterfaceStatesConfig,
    discrete: &DiscreteInterfaceStatesConfig,
) -> Vec<u32> {
    let mut ids: Vec<u32> = continuous
        .interface_id
        .iter()
        .chain(discrete.interface_id.iter())
        .copied()
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

pub fn define_capture_cross_section(
    continuous: &ContinuousInterfaceStatesConfig,
    discrete: &DiscreteInterfaceStatesConfig,
) -> CaptureCrossSectionConfig {
    let mut config = CaptureCrossSectionConfig {
        interface_id: vec![],
        model: vec![],
    };

    let interface_ids = collect_interface_ids(continuous, discrete);
    if interface_ids.is_empty() {
        return config;
    }

    println!("Define capture cross-section parameters.");
    for &id in &interface_ids {
        println!("Interface {}:", id);
        let model = get_capture_cross_section_model(id);
        config.interface_id.push(id);
        config.model.push(model);
    }

    config
}

fn get_capture_cross_section_model(interface_id: u32) -> CaptureCrossSectionModel {
    loop {
        let input = get_input(&format!(
            "Select capture cross-section model for interface {}: Constant (c) or Energy-dependent (e): default is c ",
            interface_id
        ));
        match input.trim().to_lowercase().as_str() {
            "c" | "" => {
                let sigma_cm2: f64 = get_parsed_input_with_default_nonnegative(
                    &format!("Enter sigma (cm^2) for interface {}: default is 1e-16 ", interface_id),
                    1e-16,
                );
                return CaptureCrossSectionModel::Constant {
                    sigma: sigma_cm2 * CM_TO_M.powi(2),
                };
            }
            "e" => {
                let sigma_mid_cm2: f64 = get_parsed_input_with_default_nonnegative(
                    &format!("Enter sigma_mid (cm^2) for interface {}: default is 1e-16 ", interface_id),
                    1e-16,
                );
                let e_mid: f64 = get_parsed_input_with_default_nonnegative(
                    &format!("Enter E_mid (eV) for interface {}: default is 0.5 ", interface_id),
                    0.5,
                );
                let e_slope: f64 = get_parsed_input_with_default(
                    &format!("Enter E_slope (eV) for interface {}: default is 0.1 ", interface_id),
                    0.1,
                );
                return CaptureCrossSectionModel::EnergyDependent {
                    sigma_mid: sigma_mid_cm2 * CM_TO_M.powi(2),
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
                vec![DiscreteModel::new(1.0, 0.5, 0.1, DiscreteStateType::DonorLike, 1.0)];
                n
            ],
        }
    }

    #[test]
    fn test_constant_model_stores_sigma_in_si_units() {
        let sigma_cm2 = 1e-16_f64;
        let sigma_m2 = sigma_cm2 * CM_TO_M.powi(2);
        let model = CaptureCrossSectionModel::Constant { sigma: sigma_m2 };
        match model {
            CaptureCrossSectionModel::Constant { sigma } => {
                assert!((sigma - 1e-20).abs() < 1e-30);
            }
            _ => panic!("Expected Constant"),
        }
    }

    #[test]
    fn test_energy_dependent_model_stores_sigma_mid_in_si_units() {
        let sigma_mid_cm2 = 1e-16_f64;
        let sigma_mid_m2 = sigma_mid_cm2 * CM_TO_M.powi(2);
        let model = CaptureCrossSectionModel::EnergyDependent {
            sigma_mid: sigma_mid_m2,
            e_mid: 0.5,
            e_slope: 0.1,
        };
        match model {
            CaptureCrossSectionModel::EnergyDependent { sigma_mid, e_mid, e_slope } => {
                assert!((sigma_mid - 1e-20).abs() < 1e-30);
                assert!((e_mid - 0.5).abs() < 1e-10);
                assert!((e_slope - 0.1).abs() < 1e-10);
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
