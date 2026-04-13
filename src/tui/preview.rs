use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::config::structure::MaterialType;
use crate::tui::forms::{render_field, Field, measurement::MeasurementForm, sim_settings::SimSettingsForm, structure::StructureForm};

pub fn render_settings_preview(frame: &mut Frame, form: &SimSettingsForm, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Preview")
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let parallel = if form.fields[4].bool_value() { "yes" } else { "no" };
    let lines = vec![
        Line::from(Span::styled("Simulation Settings", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(format!("  Name        {}", form.fields[0].text_value())),
        Line::from(format!("  SOR factor  {}", form.fields[1].text_value())),
        Line::from(format!("  Convergence {} eV", form.fields[2].text_value())),
        Line::from(format!("  Max iter    {}", form.fields[3].text_value())),
        Line::from(format!("  Parallel    {}", parallel)),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn render_structure_preview(frame: &mut Frame, form: &StructureForm, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Device Stack")
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 || form.layers.is_empty() {
        frame.render_widget(Paragraph::new("  No layers defined"), inner);
        return;
    }

    let bar_width = ((inner.width as usize).saturating_sub(22)).clamp(8, 16);

    let thicknesses: Vec<f64> = form
        .layers
        .iter()
        .map(|l| l.fields[2].text_value().trim().parse::<f64>().unwrap_or(100.0))
        .collect();
    let total: f64 = thicknesses.iter().sum();

    // Reserve 2 lines for Gate / Substrate labels
    let available = (inner.height as usize).saturating_sub(2);

    let mut lines: Vec<Line> = vec![];

    // Gate label
    lines.push(Line::from(Span::styled(
        format!("  {:\u{2500}<width$}", "Gate", width = bar_width + 2),
        Style::default().fg(Color::Yellow),
    )));

    for (i, layer) in form.layers.iter().enumerate() {
        if lines.len() >= inner.height as usize - 1 {
            break;
        }

        let mat = layer.material_type();
        let name = layer.fields[0].text_value();
        let thickness_nm = thicknesses[i];

        // Proportional height: at least 1 row per layer
        let height = if total == 0.0 {
            1
        } else {
            ((thickness_nm / total * available as f64).round() as usize).max(1)
        };

        let (fill_char, color) = match mat {
            MaterialType::Insulator => ('\u{2588}', Color::Blue),
            MaterialType::Semiconductor => ('\u{2591}', Color::Green),
        };

        let bar: String = std::iter::repeat(fill_char).take(bar_width).collect();
        for row in 0..height {
            if lines.len() >= inner.height as usize - 1 {
                break;
            }
            if row == height / 2 {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {}", bar), Style::default().fg(color)),
                    Span::raw(format!("  {} ({:.0}nm)", name, thickness_nm)),
                ]));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("  {}", bar),
                    Style::default().fg(color),
                )));
            }
        }
    }

    // Substrate label
    lines.push(Line::from(Span::styled(
        format!("  {:\u{2500}<width$}  Substrate", "", width = bar_width),
        Style::default().fg(Color::DarkGray),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn render_measurement_preview(frame: &mut Frame, form: &MeasurementForm, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Voltage Sweep")
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let v_start = form.fields[1].text_value();
    let v_end = form.fields[2].text_value();
    let v_step = form.fields[3].text_value();
    let temp = form.fields[0].text_value();
    let v_ac = form.fields[4].text_value();

    let arrow_len = (inner.width as usize).saturating_sub(4).max(6);
    let dashes: String = std::iter::repeat('\u{2500}').take(arrow_len - 1).collect();
    let arrow = format!("{}\u{2192}", dashes);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}V", v_start),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            format!("  {}", arrow),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!("  {}V", v_end),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(format!("  Step   {} V", v_step)),
        Line::from(format!("  VAC    {} mV", v_ac)),
        Line::from(format!("  Temp   {} K", temp)),
    ];

    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn render_run_preview(
    frame: &mut Frame,
    barrier_field: &Field,
    errors: &[String],
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Run")
        .border_style(Style::default().fg(Color::Green));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let [field_area, hint_area, errors_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .areas(inner);

    render_field(frame, barrier_field, true, field_area);

    frame.render_widget(
        Paragraph::new("\n  Press [r] or [Enter] to run\n  Press [q] to quit")
            .style(Style::default().fg(Color::Green)),
        hint_area,
    );

    if !errors.is_empty() {
        let error_lines: Vec<Line> = errors
            .iter()
            .map(|e| {
                Line::from(Span::styled(
                    format!("  \u{2717} {}", e),
                    Style::default().fg(Color::Red),
                ))
            })
            .collect();
        frame.render_widget(Paragraph::new(error_lines), errors_area);
    }
}

pub fn render_run_summary(frame: &mut Frame, app: &super::App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Configuration Summary")
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from(Span::styled(
            "[ Settings ]",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("  Name  {}", app.sim_form.fields[0].text_value())),
        Line::from(format!(
            "  SOR {}  Conv {}",
            app.sim_form.fields[1].text_value(),
            app.sim_form.fields[2].text_value()
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[ Structure ]",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
    ];

    for (i, layer) in app.struct_form.layers.iter().enumerate() {
        let mat = if layer.material_type() == MaterialType::Semiconductor {
            "Semi"
        } else {
            "Ins"
        };
        lines.push(Line::from(format!(
            "  {}  {} ({}, {}nm)",
            i,
            layer.fields[0].text_value(),
            mat,
            layer.fields[2].text_value()
        )));
    }

    lines.extend([
        Line::from(""),
        Line::from(Span::styled(
            "[ Measurement ]",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "  V  {} \u{2192} {} V  (step {}V)",
            app.measure_form.fields[1].text_value(),
            app.measure_form.fields[2].text_value(),
            app.measure_form.fields[3].text_value(),
        )),
        Line::from(format!(
            "  T  {} K   VAC {} mV",
            app.measure_form.fields[0].text_value(),
            app.measure_form.fields[4].text_value(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[ Advanced (auto defaults) ]",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  Mesh: 0.1nm step, 1 region",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  Fixed charges: none",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  Interface states: none",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    frame.render_widget(Paragraph::new(lines), inner);
}
