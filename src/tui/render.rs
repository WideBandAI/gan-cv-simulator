use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::app::{App, accumulated_mesh_nm, compute_equilibrium, total_device_nm};
use super::types::{CsModelType, EcEfMode, Page, active_interface_indices};
use crate::physics_equations::interface_states::DiscreteStateType;

// ─── Top-level draw ───────────────────────────────────────────────────────────

pub(crate) fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(frame, chunks[0], app);
    draw_content(frame, chunks[1], app);
    draw_error(frame, chunks[2], app);
    draw_help(frame, chunks[3], app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = match &app.page {
        Page::SimSettings => " [1/9] Simulation Settings ".to_string(),
        Page::Measurement => " [2/9] Measurement ".to_string(),
        Page::StructureCount => " [3/9] Device Structure ".to_string(),
        Page::Layer(i) => format!(" [3/9] Layer {} of {} ", i + 1, app.layers.len()),
        Page::MeshCount => " [4/9] Mesh Settings ".to_string(),
        Page::MeshLayer(i) => {
            format!(" [4/9] Mesh Layer {} of {} ", i + 1, app.mesh_layers.len())
        }
        Page::FixedCharge => " [5/9] Fixed Charges ".to_string(),
        Page::InterfaceStates(i) => {
            format!(
                " [6/9] Interface {}/{} ",
                i + 1,
                app.layers.len().saturating_sub(1)
            )
        }
        Page::DiscreteState(i, j) => {
            let left = &app.layers[*i].name;
            let right = &app.layers[*i + 1].name;
            format!(" [6/9] {left}/{right} - Discrete Trap {} ", j + 1)
        }
        Page::CaptureCrossSection(k) => {
            let active = active_interface_indices(&app.interface_states);
            format!(" [7/9] Capture Cross Section {}/{} ", k + 1, active.len())
        }
        Page::BoundaryConditions => " [8/9] Boundary Conditions ".to_string(),
        Page::Confirm => " [9/9] Confirm & Run ".to_string(),
    };
    let header = Paragraph::new("  GaN C-V Simulator")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(header, area);
}

fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    match &app.page {
        Page::SimSettings => draw_sim_settings(frame, area, app),
        Page::Measurement => draw_measurement(frame, area, app),
        Page::StructureCount => draw_structure_count(frame, area, app),
        Page::Layer(i) => draw_layer(frame, area, app, *i),
        Page::MeshCount => draw_mesh_count(frame, area, app),
        Page::MeshLayer(i) => draw_mesh_layer(frame, area, app, *i),
        Page::FixedCharge => draw_fixed_charge(frame, area, app),
        Page::InterfaceStates(i) => draw_interface_state(frame, area, app, *i),
        Page::DiscreteState(i, j) => draw_discrete_state(frame, area, app, *i, *j),
        Page::CaptureCrossSection(k) => draw_capture_cross_section(frame, area, app, *k),
        Page::BoundaryConditions => draw_boundary_conditions(frame, area, app),
        Page::Confirm => draw_confirm(frame, area, app),
    }
}

fn draw_error(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(err) = &app.error {
        let msg = Paragraph::new(format!(" ✖ {err}"))
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        frame.render_widget(msg, area);
    }
}

fn draw_help(frame: &mut Frame, area: Rect, app: &App) {
    let text = if app.page == Page::Confirm {
        " Enter: Run  Esc: Back  Ctrl+C: Quit"
    } else {
        " Tab/↓: Next  ↑: Prev  Space: Toggle  Enter/→: Next page  Esc/←: Back  Ctrl+C: Quit"
    };
    let help = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, area);
}

// ─── Field rendering helper ───────────────────────────────────────────────────

/// Each field: (label, current_value_string, is_toggle)
fn field_lines(fields: &[(&str, String, bool)], focused: usize) -> Vec<Line<'static>> {
    fields
        .iter()
        .enumerate()
        .map(|(i, (label, value, is_toggle))| {
            let active = i == focused;
            let prefix = if active { "> " } else { "  " };
            let val_display = if *is_toggle {
                format!("[ {} ]", value)
            } else if active {
                format!("{}█", value)
            } else {
                value.clone()
            };
            let style = if active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(
                format!("{prefix}{label:<34}{val_display}"),
                style,
            ))
        })
        .collect()
}

// ─── Per-page draw functions ──────────────────────────────────────────────────

fn draw_sim_settings(frame: &mut Frame, area: Rect, app: &App) {
    let fields = [
        ("Simulation Name", app.sim_name.clone(), false),
        ("SOR Relaxation Factor", app.sor_factor.clone(), false),
        ("Convergence Criterion (eV)", app.convergence.clone(), false),
        ("Max Iterations", app.max_iter.clone(), false),
        (
            "Parallel Processing",
            if app.parallel {
                "ON".to_string()
            } else {
                "OFF".to_string()
            },
            true,
        ),
    ];
    let lines = field_lines(&fields, app.focused);
    let para =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Fields "));
    frame.render_widget(para, area);
}

fn draw_measurement(frame: &mut Frame, area: Rect, app: &App) {
    let fields = [
        ("Temperature (K)", app.temperature.clone(), false),
        ("Voltage Start (V)", app.v_start.clone(), false),
        ("Voltage End (V)", app.v_end.clone(), false),
        ("Voltage Step (V)", app.v_step.clone(), false),
        ("AC Voltage (mV)", app.ac_voltage.clone(), false),
        ("Measurement Time (s)", app.meas_time.clone(), false),
        ("Stress Voltage (V)", app.stress_voltage.clone(), false),
        (
            "Stress Relief Voltage (V)",
            app.stress_relief_voltage.clone(),
            false,
        ),
        (
            "Stress Relief Time (s)",
            app.stress_relief_time.clone(),
            false,
        ),
    ];
    let lines = field_lines(&fields, app.focused);
    let para =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Fields "));
    frame.render_widget(para, area);
}

fn draw_structure_count(frame: &mut Frame, area: Rect, app: &App) {
    let fields = [("Number of Layers", app.num_layers_str.clone(), false)];
    let lines = field_lines(&fields, app.focused);
    let para =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Fields "));
    frame.render_widget(para, area);
}

fn draw_layer(frame: &mut Frame, area: Rect, app: &App, i: usize) {
    let layer = &app.layers[i];
    let is_last = i + 1 == app.layers.len();
    let mat_val = if layer.is_semiconductor() {
        "Semiconductor"
    } else {
        "Insulator"
    };
    let mut fields: Vec<(&str, String, bool)> = vec![
        ("Name", layer.name.clone(), false),
        ("Material Type [Space: toggle]", mat_val.to_string(), true),
        ("Thickness (nm)", layer.thickness_nm.clone(), false),
        ("Relative Permittivity", layer.permittivity.clone(), false),
        ("Bandgap Energy (eV)", layer.bandgap_ev.clone(), false),
    ];
    if !is_last {
        fields.push((
            "Delta Conduction Band (eV)",
            layer.delta_cb_ev.clone(),
            false,
        ));
    }
    if layer.is_semiconductor() {
        fields.push((
            "Effective Mass Coeff",
            layer.mass_electron_coeff.clone(),
            false,
        ));
        fields.push((
            "Donor Concentration (cm⁻³)",
            layer.donor_conc_cm3.clone(),
            false,
        ));
        fields.push((
            "Energy Level Donor Ec-Ed (eV)",
            layer.energy_donor_ev.clone(),
            false,
        ));
    }
    let lines = field_lines(&fields, app.focused);
    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Layer {i} ")),
    );
    frame.render_widget(para, area);
}

fn draw_mesh_count(frame: &mut Frame, area: Rect, app: &App) {
    let total_nm = total_device_nm(app);
    let info = match total_nm {
        Some(t) => format!(
            "\n  Total device thickness: {t:.3} nm\n  Fixed charges and interface states default to zero/none."
        ),
        None => "\n  (device thickness not yet available)".to_string(),
    };
    let fields = [
        (
            "Number of Mesh Layers",
            app.num_mesh_layers_str.clone(),
            false,
        ),
        ("Energy Step (meV)", app.energy_step_mev.clone(), false),
    ];
    let lines = field_lines(&fields, app.focused);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(4)])
        .split(area);
    let para =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Fields "));
    frame.render_widget(para, chunks[0]);
    let note = Paragraph::new(info)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Info "));
    frame.render_widget(note, chunks[1]);
}

fn draw_mesh_layer(frame: &mut Frame, area: Rect, app: &App, i: usize) {
    let ml = &app.mesh_layers[i];
    let is_last = i + 1 == app.mesh_layers.len();

    let mut fields: Vec<(&str, String, bool)> =
        vec![("Mesh Length (nm)", ml.mesh_length_nm.clone(), false)];
    if !is_last {
        fields.push(("Thickness (nm)", ml.thickness_nm.clone(), false));
    }

    // Info: show auto-calculated thickness for the last layer
    let info = if is_last {
        let total_nm = total_device_nm(app).unwrap_or(0.0);
        let acc_nm = accumulated_mesh_nm(app, i).unwrap_or(0.0);
        let remaining = total_nm - acc_nm;
        format!(
            "\n  Auto thickness (remaining): {remaining:.3} nm\n  (= total {total_nm:.3} nm - accumulated {acc_nm:.3} nm)"
        )
    } else {
        let total_nm = total_device_nm(app).unwrap_or(0.0);
        let acc_nm = accumulated_mesh_nm(app, i).unwrap_or(0.0);
        let after = accumulated_mesh_nm(app, i + 1).unwrap_or(0.0);
        format!(
            "\n  Accumulated so far: {acc_nm:.3} nm  →  after this layer: {after:.3} nm\n  Total device: {total_nm:.3} nm"
        )
    };

    let lines = field_lines(&fields, app.focused);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(4)])
        .split(area);
    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Mesh Layer {i} ")),
    );
    frame.render_widget(para, chunks[0]);
    let note = Paragraph::new(info)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Info "));
    frame.render_widget(note, chunks[1]);
}

fn draw_fixed_charge(frame: &mut Frame, area: Rect, app: &App) {
    let n = app.layers.len();
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Section header: bulk charges
    lines.push(Line::from(Span::styled(
        "  Bulk Fixed Charge (C/cm\u{00b3}):".to_string(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    for (i, val) in app.bulk_charge_densities.iter().enumerate() {
        let active = app.focused == i;
        let prefix = if active { "> " } else { "  " };
        let label = format!("Layer '{}'", app.layers[i].name);
        let val_display = if active {
            format!("{val}\u{2588}")
        } else {
            val.clone()
        };
        let style = if active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled(
            format!("{prefix}{label:<34}{val_display}"),
            style,
        )));
    }

    // Section header: interface charges (only if ≥ 2 layers)
    if n >= 2 {
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "  Interface Fixed Charge (C/cm\u{00b2}):".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        for (i, val) in app.interface_charge_densities.iter().enumerate() {
            let field_idx = n + i;
            let active = app.focused == field_idx;
            let prefix = if active { "> " } else { "  " };
            let label = format!("{}/{}", app.layers[i].name, app.layers[i + 1].name);
            let val_display = if active {
                format!("{val}\u{2588}")
            } else {
                val.clone()
            };
            let style = if active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!("{prefix}{label:<34}{val_display}"),
                style,
            )));
        }
    }

    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Fixed Charges (default: 0) "),
    );
    frame.render_widget(para, area);
}

fn draw_interface_state(frame: &mut Frame, area: Rect, app: &App, i: usize) {
    let ist = &app.interface_states[i];
    let left_name = &app.layers[i].name;
    let right_name = &app.layers[i + 1].name;

    let cont_str = if ist.has_continuous { "ON" } else { "OFF" };
    let disc_str = if ist.has_discrete { "ON" } else { "OFF" };

    let mut fields: Vec<(&str, String, bool)> = vec![(
        "Continuous Traps [Space: toggle]",
        cont_str.to_string(),
        true,
    )];
    if ist.has_continuous {
        let c = &ist.continuous;
        fields.push(("Dit0 (cm\u{207b}\u{00b2})", c.dit0.clone(), false));
        fields.push(("nssec", c.nssec.clone(), false));
        fields.push(("nssev", c.nssev.clone(), false));
        fields.push(("|Ec \u{2212} Ecnl| (eV)", c.ecnl.clone(), false));
        fields.push(("nd", c.nd.clone(), false));
        fields.push(("na", c.na.clone(), false));
    }
    fields.push((
        "Discrete Traps  [Space: toggle]",
        disc_str.to_string(),
        true,
    ));
    if ist.has_discrete {
        fields.push((
            "Number of Discrete Traps",
            ist.num_discrete_str.clone(),
            false,
        ));
    }

    let lines = field_lines(&fields, app.focused);
    let title = format!(" Interface {left_name}/{right_name} ");
    let para = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(para, area);
}

fn draw_discrete_state(frame: &mut Frame, area: Rect, app: &App, i: usize, j: usize) {
    let trap = &app.interface_states[i].discrete_traps[j];
    let state_str = match trap.state_type {
        DiscreteStateType::DonorLike => "DonorLike",
        DiscreteStateType::AcceptorLike => "AcceptorLike",
    };
    let fields = [
        ("Ditmax (cm\u{207b}\u{00b2})", trap.ditmax.clone(), false),
        ("|Ec \u{2212} Ed| (eV)", trap.ed.clone(), false),
        ("FWHM (eV)", trap.fwhm.clone(), false),
        ("State Type [Space: toggle]", state_str.to_string(), true),
    ];
    let left_name = &app.layers[i].name;
    let right_name = &app.layers[i + 1].name;
    let title = format!(
        " {left_name}/{right_name} \u{2013} Trap {} of {} ",
        j + 1,
        app.interface_states[i].discrete_traps.len()
    );
    let lines = field_lines(&fields, app.focused);
    let para = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(para, area);
}

fn draw_capture_cross_section(frame: &mut Frame, area: Rect, app: &App, k: usize) {
    let active = active_interface_indices(&app.interface_states);
    let Some(&iface_id) = active.get(k) else {
        return;
    };
    let left_name = &app.layers[iface_id].name;
    let right_name = &app.layers[iface_id + 1].name;
    let ccs = &app.capture_cross_sections[k];

    let model_str = match ccs.model_type {
        CsModelType::Constant => "Constant",
        CsModelType::EnergyDependent => "EnergyDependent",
    };

    let mut fields: Vec<(&str, String, bool)> =
        vec![("Model [Space: toggle]", model_str.to_string(), true)];
    match ccs.model_type {
        CsModelType::Constant => {
            fields.push(("Sigma (cm\u{00b2})", ccs.sigma.clone(), false));
        }
        CsModelType::EnergyDependent => {
            fields.push(("Sigma_mid (cm\u{00b2})", ccs.sigma_mid.clone(), false));
            fields.push(("E_mid (eV)", ccs.e_mid.clone(), false));
            fields.push(("E_slope (eV)", ccs.e_slope.clone(), false));
        }
    }
    fields.push((
        "Effective Mass Coeff",
        ccs.mass_electron_coeff.clone(),
        false,
    ));

    let lines = field_lines(&fields, app.focused);
    let title = format!(
        " {left_name}/{right_name} \u{2013} Capture Cross Section ({}/{}) ",
        k + 1,
        active.len()
    );
    let para = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(para, area);
}

fn draw_boundary_conditions(frame: &mut Frame, area: Rect, app: &App) {
    let bottom_is_sc = app
        .layers
        .last()
        .map(|l| l.is_semiconductor())
        .unwrap_or(false);

    let mut fields: Vec<(&str, String, bool)> =
        vec![("Barrier Height (eV)", app.barrier_height_ev.clone(), false)];

    if bottom_is_sc {
        let mode_label = match app.ec_ef_mode {
            EcEfMode::Manual => "Manual".to_string(),
            EcEfMode::Equilibrium => match compute_equilibrium(app) {
                Some(v) => format!("Equilibrium ({:.4} eV)", v),
                None => "Equilibrium (incomplete layer data)".to_string(),
            },
        };
        fields.push(("Ec-Ef Source [Space: toggle]", mode_label, true));
        if app.ec_ef_mode == EcEfMode::Manual {
            fields.push(("Ec - Ef Bottom (eV)", app.ec_ef_bottom_ev.clone(), false));
        }
    } else {
        fields.push(("Ec - Ef Bottom (eV)", app.ec_ef_bottom_ev.clone(), false));
    }

    let lines = field_lines(&fields, app.focused);
    let para =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Fields "));
    frame.render_widget(para, area);
}

fn draw_confirm(frame: &mut Frame, area: Rect, app: &App) {
    let layer_summary: String = app
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let mat = if l.is_semiconductor() { "SC" } else { "Ins" };
            format!("  Layer {i}: {} ({}) {}nm\n", l.name, mat, l.thickness_nm)
        })
        .collect();

    let mesh_summary: String = {
        let total_nm = total_device_nm(app).unwrap_or(0.0);
        let nm = app.mesh_layers.len();
        let mut acc = 0.0_f64;
        app.mesh_layers
            .iter()
            .enumerate()
            .map(|(i, ml)| {
                if i == nm - 1 {
                    let remaining = total_nm - acc;
                    format!(
                        "  Mesh {i}: length={}nm  thickness={:.3}nm (auto)\n",
                        ml.mesh_length_nm, remaining
                    )
                } else {
                    let t: f64 = ml.thickness_nm.trim().parse().unwrap_or(0.0);
                    acc += t;
                    format!(
                        "  Mesh {i}: length={}nm  thickness={}nm\n",
                        ml.mesh_length_nm, ml.thickness_nm
                    )
                }
            })
            .collect()
    };

    let ec_ef_str = match app.ec_ef_mode {
        EcEfMode::Manual => app.ec_ef_bottom_ev.clone(),
        EcEfMode::Equilibrium => match compute_equilibrium(app) {
            Some(v) => format!("{v:.4} (equilibrium)"),
            None => "? (equilibrium)".to_string(),
        },
    };

    let summary = format!(
        "\n  Simulation:   {}\n  Temperature:  {} K\n  Voltage:      {} → {} V  (step {} V)\n  AC Voltage:   {} mV\n\n{}\n  Energy step:  {} meV\n{}\n  Barrier:      {} eV\n  Ec-Ef bottom: {} eV\n\n  Fixed charges and interface states: all zero / none (default)\n\n  Press Enter to start simulation.",
        app.sim_name,
        app.temperature,
        app.v_start,
        app.v_end,
        app.v_step,
        app.ac_voltage,
        layer_summary,
        app.energy_step_mev,
        mesh_summary,
        app.barrier_height_ev,
        ec_ef_str,
    );
    let para = Paragraph::new(summary)
        .block(Block::default().borders(Borders::ALL).title(" Summary "))
        .style(Style::default().fg(Color::Green));
    frame.render_widget(para, area);
}
