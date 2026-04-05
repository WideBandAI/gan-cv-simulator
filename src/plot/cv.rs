use crate::plot::style::mesh_style;
use crate::plot::utils::find_range;
use plotters::prelude::*;

pub fn plot_cv_curves(
    voltage: &[f64],
    capacitance: &[f64],
    filename: &str,
    save_dir: &str,
) -> anyhow::Result<()> {
    let save_dir_path = std::path::Path::new(save_dir);
    if save_dir_path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        anyhow::bail!("Invalid save directory: contains path traversal components.");
    }

    if filename.contains(['/', '\\']) {
        anyhow::bail!("Invalid filename: must not contain path separators.");
    }

    let filepath = save_dir_path.join(filename);
    let root = BitMapBackend::new(&filepath, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let (xmin, xmax) = find_range(voltage);
    let (ymin, ymax) = find_range(capacitance);

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
