use crate::solvers::cv_solver::CVResult;
use crate::utils::anti_traversal_filename;
use std::fs;
use std::io::Write;

pub fn save_cv_curves(
    cv_results: &[CVResult],
    save_dir: &str,
    filename: &str,
) -> anyhow::Result<()> {
    super::validate_save_dir(save_dir)?;
    let save_dir_path = std::path::Path::new(save_dir);

    let filename = match anti_traversal_filename(filename) {
        Some(name) => name,
        None => {
            anyhow::bail!("Invalid filename: must not contain path separators or '..'.");
        }
    };

    let cv_file_path = save_dir_path.join(&filename);
    fs::create_dir_all(save_dir_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create output directory '{}': {}. Please check permissions and try again.",
            save_dir_path.display(),
            e
        )
    })?;

    let mut file = std::fs::File::create(&cv_file_path)
        .map_err(|e| anyhow::anyhow!("Failed to create C-V curve file '{:?}': {}", filename, e))?;

    writeln!(file, "Gate Voltage (V), Capacitance (nF/cm^2)")?;
    for result in cv_results {
        for (gate_voltage, capacitance) in result.gate_voltage.iter().zip(result.capacitance.iter())
        {
            writeln!(file, "{:.3}, {:.3e}", gate_voltage, capacitance)?;
        }
    }

    Ok(())
}
