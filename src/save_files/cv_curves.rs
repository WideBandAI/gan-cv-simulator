use crate::solvers::cv_solver::CVResult;
use std::fs;

pub fn save_cv_curves(
    cv_results: &[CVResult],
    save_dir: &str,
    filename: &str,
) -> anyhow::Result<()> {
    // Guard against path traversal by disallowing `..` components.
    // We allow absolute paths as tempfile::TempDir generates them.
    let save_dir_path = std::path::Path::new(save_dir);
    if save_dir_path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        anyhow::bail!("Invalid save directory: contains path traversal components.");
    }

    let filename = match std::path::Path::new(filename).file_name() {
        Some(name) if name == std::path::Path::new(filename) => name,
        _ => {
            anyhow::bail!("Invalid filename: must not contain path separators.");
        }
    };

    let cv_save_dir = save_dir_path.join("cv_curves");
    let cv_file_path = cv_save_dir.join(filename);
    fs::create_dir_all(&cv_save_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create output directory '{}': {}. Please check permissions and try again.",
            cv_save_dir.display(),
            e
        )
    })?;

    let mut file = std::fs::File::create(&cv_file_path)
        .map_err(|e| anyhow::anyhow!("Failed to create C-V curve file '{:?}': {}", filename, e))?;

    writeln!(file, "Gate Voltage (V), Capacitance (nF/cm^2)")?;
    for result in cv_results {
        writeln!(
            file,
            "{:.3}, {:.3e}",
            result.gate_voltage, result.capacitance
        )?;
    }

    Ok(())
}
