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

    let save_dir_path = std::path::Path::new(save_dir);
    let file_path = save_dir_path.join(filename);

    let root = BitMapBackend::new(&file_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let (xmin, xmax) = find_range(voltage);
    let (ymin, ymax) = find_range(capacitance);

    let mut chart = ChartBuilder::on(&root)
        .margin(30)
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d(xmin..xmax, ymin..ymax)?;

    chart
        .configure_mesh()
        .x_desc("Gate Voltage (V)")
        .y_desc("Capacitance (F/cm²)")
        .axis_desc_style(("sans-serif", 22))
        .label_style(("sans-serif", 18))
        // 軸
        .axis_style(BLACK.stroke_width(2))
        // grid (major)
        .bold_line_style(RGBColor(200, 200, 200))
        // grid (minor)
        .light_line_style(RGBColor(230, 230, 230))
        .draw()?;

    chart.draw_series(LineSeries::new(
        voltage.iter().zip(capacitance).map(|(&v, &c)| (v, c)),
        RED.stroke_width(3),
    ))?;

    root.present()?;

    Ok(())
}
