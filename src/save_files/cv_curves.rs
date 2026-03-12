use crate::solvers::cv_solver::CVResult;
use plotters::prelude::*;
use std::fs;
use std::io::Write;

use crate::utils::find_range;

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

    let cv_file_path = save_dir_path.join(filename);
    fs::create_dir_all(&save_dir_path).map_err(|e| {
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

pub fn plot_cv_curves(
    cv_results: &[CVResult],
    save_dir: &str,
    filename: &str,
) -> anyhow::Result<()> {
    let voltage = &cv_results[0].gate_voltage;
    let capacitance = &cv_results[0].capacitance;

    let file_path = std::path::Path::new(save_dir).join(filename);
    let _ = match file_path.file_name() {
        Some(name) if name == file_path => name,
        _ => {
            anyhow::bail!("Invalid filename: must not contain path separators.");
        }
    };
    let root = BitMapBackend::new(&file_path, (900, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let (xmin, xmax) = find_range(voltage);
    let (ymin, ymax) = find_range(capacitance);

    let mut chart = ChartBuilder::on(&root)
        .caption("C-V curve", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(xmin..xmax, ymin..ymax)?;

    chart
        .configure_mesh()
        .x_desc("Gate Voltage (V)")
        .y_desc("Capacitance (F/cm^2)")
        .light_line_style(&RGBColor(220, 220, 220))
        .draw()?;

    chart.draw_series(LineSeries::new(
        voltage.iter().zip(capacitance).map(|(&v, &c)| (v, c)),
        RED.stroke_width(3),
    ))?;

    root.present()?;
    Ok(())
}
