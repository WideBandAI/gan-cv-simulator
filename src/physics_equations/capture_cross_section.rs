use crate::config::capture_cross_section::CaptureCrossSectionModel;

/// capture_cross_section_distribution the capture cross-section [m²] for the given model and trap energy.
///
/// # Arguments
///
/// - `model` - The capture cross-section model.
/// - `energy` (`f64`) - Trap energy level |E - Ec| in eV.
///
/// # Returns
///
/// - `f64` - Capture cross-section in m².
pub fn capture_cross_section_distribution(model: &CaptureCrossSectionModel, energy: f64) -> f64 {
    match model {
        CaptureCrossSectionModel::Constant { sigma } => *sigma,
        CaptureCrossSectionModel::EnergyDependent {
            sigma_mid,
            e_mid,
            e_slope,
        } => {
            if e_slope.abs() < f64::EPSILON {
                if (energy - e_mid).abs() < f64::EPSILON {
                    *sigma_mid
                } else {
                    0.0
                }
            } else {
                sigma_mid * ((energy - e_mid) / e_slope).exp()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::units::CM_TO_M;

    #[test]
    fn test_constant_model_returns_sigma_regardless_of_energy() {
        let sigma_m2 = 1e-16_f64 * CM_TO_M.powi(2);
        let model = CaptureCrossSectionModel::Constant { sigma: sigma_m2 };
        assert_eq!(capture_cross_section_distribution(&model, 0.0), sigma_m2);
        assert_eq!(capture_cross_section_distribution(&model, 1.0), sigma_m2);
        assert_eq!(capture_cross_section_distribution(&model, 3.3), sigma_m2);
    }

    #[test]
    fn test_energy_dependent_model_at_e_mid_returns_sigma_mid() {
        let sigma_mid = 1e-16_f64 * CM_TO_M.powi(2);
        let e_mid = 0.5;
        let e_slope = 0.1;
        let model = CaptureCrossSectionModel::EnergyDependent {
            sigma_mid,
            e_mid,
            e_slope,
        };
        // At energy == e_mid, exp(0) == 1, so result == sigma_mid
        let result = capture_cross_section_distribution(&model, e_mid);
        assert!((result - sigma_mid).abs() < 1e-40);
    }

    #[test]
    fn test_energy_dependent_model_above_e_mid_increases() {
        let sigma_mid = 1e-16_f64 * CM_TO_M.powi(2);
        let e_mid = 0.5;
        let e_slope = 0.1;
        let model = CaptureCrossSectionModel::EnergyDependent {
            sigma_mid,
            e_mid,
            e_slope,
        };
        let result_above = capture_cross_section_distribution(&model, e_mid + e_slope);
        let expected = sigma_mid * ((e_mid + e_slope - e_mid) / e_slope).exp();
        assert_eq!(result_above, expected);
        assert!(result_above > sigma_mid);
    }

    #[test]
    fn test_energy_dependent_model_below_e_mid_decreases() {
        let sigma_mid = 1e-16_f64 * CM_TO_M.powi(2);
        let e_mid = 0.5;
        let e_slope = 0.1;
        let model = CaptureCrossSectionModel::EnergyDependent {
            sigma_mid,
            e_mid,
            e_slope,
        };
        let result_below = capture_cross_section_distribution(&model, e_mid - e_slope);
        let expected = sigma_mid * ((e_mid - e_slope - e_mid) / e_slope).exp();
        assert_eq!(result_below, expected);
        assert!(result_below < sigma_mid);
    }

    #[test]
    fn test_energy_dependent_model_e_slope_zero_at_e_mid_returns_sigma_mid() {
        let sigma_mid = 1e-16_f64 * CM_TO_M.powi(2);
        let e_mid = 0.5;
        let model = CaptureCrossSectionModel::EnergyDependent {
            sigma_mid,
            e_mid,
            e_slope: 0.0,
        };
        let result = capture_cross_section_distribution(&model, e_mid);
        assert!((result - sigma_mid).abs() < 1e-40);
    }

    #[test]
    fn test_energy_dependent_model_e_slope_zero_away_from_e_mid_returns_zero() {
        let sigma_mid = 1e-16_f64 * CM_TO_M.powi(2);
        let e_mid = 0.5;
        let model = CaptureCrossSectionModel::EnergyDependent {
            sigma_mid,
            e_mid,
            e_slope: 0.0,
        };
        let result = capture_cross_section_distribution(&model, 0.0);
        assert_eq!(result, 0.0);
    }
}
