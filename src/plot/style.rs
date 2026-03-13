use plotters::coord::types::*;
use plotters::prelude::*;

pub fn mesh_style<'a, DB: DrawingBackend>(
    chart: &mut ChartContext<'a, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    x_desc: &str,
    y_desc: &str,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
    chart
        .configure_mesh()
        .x_desc(x_desc)
        .y_desc(y_desc)
        .axis_desc_style(("Arial", 22))
        .label_style(("Arial", 18))
        .axis_style(BLACK.stroke_width(2))
        .bold_line_style(RGBColor(200, 200, 200))
        .light_line_style(RGBColor(230, 230, 230))
        .draw()
}
