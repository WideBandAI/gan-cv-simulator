use crate::utils::anti_traversal_filename;
use crate::utils::get_input;
use crate::utils::get_parsed_input_with_default;

#[derive(Debug)]
pub struct SimSettings {
    pub sim_name: String,
    pub convergence_criterion: f64,
    pub max_iterations: usize,
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

    let convergence_criterion: f64 = get_parsed_input_with_default(
        "Enter the convergence criterion (max|δφ| in eV). Default is 1e-6: ",
        1e-6,
    );
    let max_iterations: usize = get_parsed_input_with_default(
        "Enter the maximum number of iterations. Default is 500000: ",
        500000,
    );

    SimSettings {
        sim_name,
        convergence_criterion,
        max_iterations,
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
