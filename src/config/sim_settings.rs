use crate::physics_equations::fermi_dirac_density::CarrierStatistics;
use crate::utils::anti_traversal_filename;
use crate::utils::get_bool_input;
use crate::utils::get_input;
use crate::utils::get_parsed_input_with_default;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SimSettings {
    pub sim_name: String,
    pub sor_relaxation_factor: f64,
    pub convergence_criterion: f64,
    pub max_iterations: usize,
    pub parallel_use: bool,
    /// Electron density statistics; omitted fields retain the legacy model.
    #[serde(default)]
    pub carrier_statistics: CarrierStatistics,
}

/// Validates a simulation name taken from user input.
///
/// - Must not be empty.
/// - Must not contain path separators (`/` or `\`).
/// - Must not contain any `..` segments (to prevent path traversal).
/// - Must consist of only ASCII alphanumerics, `_`, `-`, or `.`.
fn validate_sim_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Disallow path traversal and absolute paths.
    let _ = match anti_traversal_filename(trimmed) {
        Some(name) => name,
        None => return false,
    };

    // Allow only a limited set of safe characters.
    trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

fn get_sim_name() -> String {
    loop {
        let input = get_input("Enter a name for this simulation: ");
        if validate_sim_name(&input) {
            return input.trim().to_string();
        }

        println!(
            "Invalid simulation name. Use only letters, digits, '-', '_', or '.'. Do not include '/' or '\\' or '..'."
        );
    }
}

pub fn define_sim_settings() -> SimSettings {
    println!("Define simulation settings.");
    let sim_name: String = get_sim_name();

    let sor_relaxation_factor: f64 =
        get_parsed_input_with_default("Enter the SOR relaxation factor. Default is 1.9: ", 1.9);
    let convergence_criterion: f64 = get_parsed_input_with_default(
        "Enter the convergence criterion (in eV). Default is 1e-6: ",
        1e-6,
    );
    let max_iterations: usize = get_parsed_input_with_default(
        "Enter the maximum number of iterations. Default is 1000000: ",
        1000000,
    );
    let parallel_use: bool =
        get_bool_input("Use parallel processing for Poisson solver? (y/n). Default is n: ");

    let carrier_statistics =
        if get_bool_input("Use Fermi-Dirac electron density? (y/n). Default is n: ") {
            CarrierStatistics::FermiDirac
        } else {
            CarrierStatistics::Boltzmann
        };

    SimSettings {
        carrier_statistics,
        sim_name,
        sor_relaxation_factor,
        convergence_criterion,
        max_iterations,
        parallel_use,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_sim_name_allows_good_names() {
        assert!(validate_sim_name("my_simulation"));
        assert!(validate_sim_name("sim-1"));
        assert!(validate_sim_name("sim.1"));
        assert!(validate_sim_name("SIM_123"));
    }

    #[test]
    fn validate_sim_name_rejects_bad_names() {
        assert!(!validate_sim_name(""));
        assert!(!validate_sim_name("../etc"));
        assert!(!validate_sim_name(".."));
        assert!(!validate_sim_name("foo/bar"));
        assert!(!validate_sim_name("foo\\bar"));
        assert!(!validate_sim_name("sim name"));
    }
}

#[cfg(test)]
mod carrier_statistics_tests {
    use super::*;

    #[test]
    fn old_json_defaults_to_boltzmann_and_fd_round_trips() {
        let old = r#"{"sim_name":"legacy","sor_relaxation_factor":1.9,"convergence_criterion":1e-6,"max_iterations":1000,"parallel_use":false}"#;
        let mut settings: SimSettings = serde_json::from_str(old).unwrap();
        assert_eq!(settings.carrier_statistics, CarrierStatistics::Boltzmann);
        settings.carrier_statistics = CarrierStatistics::FermiDirac;
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("fermi_dirac"));
        let restored: SimSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.carrier_statistics, CarrierStatistics::FermiDirac);
        assert!(
            serde_json::from_str::<SimSettings>(
                &old.replace("false", "false,\"carrier_statistics\":\"typo\"")
            )
            .is_err()
        );
    }
}
