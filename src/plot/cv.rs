use crate::plot::style::mesh_style;
use crate::plot::utils::find_range;
use crate::utils::anti_traversal_filename;
use plotters::prelude::*;

pub fn plot_cv_curves(
    voltage: &[f64],
    capacitance: &[f64],
    filename: &str,
    save_dir: &str,
) -> anyhow::Result<()> {
    crate::save_files::validate_save_dir(save_dir)?;
    let save_dir_path = std::path::Path::new(save_dir);

    let filename = match anti_traversal_filename(filename) {
        Some(name) => name,
        None => {
            anyhow::bail!("Invalid filename: must not contain path separators or '..'.");
        }
    };

    let filepath = save_dir_path.join(&filename);
    let root = BitMapBackend::new(&filepath, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let (xmin, xmax) = find_range(voltage);
    let (_ymin, ymax) = find_range(capacitance);
    let ymin = _ymin.min(0.0); // Ensure y-axis starts at 0

    let mut chart = ChartBuilder::on(&root)
        .margin(30)
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d(xmin..xmax, ymin..ymax)?;

    mesh_style(&mut chart, "Gate Voltage (V)", "Capacitance (nF/cm²)")?;

    chart.draw_series(LineSeries::new(
        voltage.iter().zip(capacitance).map(|(&v, &c)| (v, c)),
        RED.stroke_width(3),
    ))?;

    root.present()?;
    Ok(())
}
