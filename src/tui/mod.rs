use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

use crate::config::{
    boundary_conditions::BoundaryConditions,
    capture_cross_section::CaptureCrossSectionConfig,
    configuration_builder::{Configuration, ConfigurationBuilder},
    fixcharge::{BulkFixedCharge, InterfaceFixedCharge},
    interface_states::{ContinuousInterfaceStatesConfig, DiscreteInterfaceStatesConfig},
    measurement::{Measurement, Stress, Temperature, Time, Voltage},
    mesh::MeshParams,
    sim_settings::SimSettings,
    structure::{DeviceStructure, MaterialType},
};
use crate::constants::physics::{EPSILON_0, M_ELECTRON};
use crate::constants::units::{MEV_TO_EV, MV_TO_V, NM_TO_M, PER_CM3_TO_PER_M3};

// ─── Pages ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Page {
    SimSettings,
    Measurement,
    StructureCount,
    Layer(usize),
    MeshBoundary,
    Confirm,
}

// ─── Layer input state ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct LayerInput {
    name: String,
    material: MaterialType,
    thickness_nm: String,
    permittivity: String,
    bandgap_ev: String,
    delta_cb_ev: String,
    mass_electron_coeff: String,
    donor_conc_cm3: String,
    energy_donor_ev: String,
}

impl LayerInput {
    fn new(index: usize) -> Self {
        Self {
            name: format!("layer_{index}"),
            material: MaterialType::Semiconductor,
            thickness_nm: String::new(),
            permittivity: String::new(),
            bandgap_ev: String::new(),
            delta_cb_ev: "0.0".to_string(),
            mass_electron_coeff: "0.2".to_string(),
            donor_conc_cm3: "1e16".to_string(),
            energy_donor_ev: "0.025".to_string(),
        }
    }

    fn is_semiconductor(&self) -> bool {
        self.material == MaterialType::Semiconductor
    }

    fn field_count(&self) -> usize {
        if self.is_semiconductor() { 9 } else { 6 }
    }
}

// ─── App state ────────────────────────────────────────────────────────────────

struct App {
    page: Page,
    focused: usize,
    error: Option<String>,

    // SimSettings
    sim_name: String,
    sor_factor: String,
    convergence: String,
    max_iter: String,
    parallel: bool,

    // Measurement
    temperature: String,
    v_start: String,
    v_end: String,
    v_step: String,
    ac_voltage: String,
    meas_time: String,
    stress_voltage: String,
    stress_relief_voltage: String,
    stress_relief_time: String,

    // Structure
    num_layers_str: String,
    layers: Vec<LayerInput>,

    // Mesh & Boundary
    mesh_length_nm: String,
    energy_step_mev: String,
    barrier_height_ev: String,
    ec_ef_bottom_ev: String,
}

impl App {
    fn new() -> Self {
        Self {
            page: Page::SimSettings,
            focused: 0,
            error: None,
            sim_name: String::new(),
            sor_factor: "1.9".to_string(),
            convergence: "1e-6".to_string(),
            max_iter: "100000".to_string(),
            parallel: false,
            temperature: "300".to_string(),
            v_start: String::new(),
            v_end: String::new(),
            v_step: String::new(),
            ac_voltage: "20".to_string(),
            meas_time: "100".to_string(),
            stress_voltage: "0".to_string(),
            stress_relief_voltage: "0".to_string(),
            stress_relief_time: "0".to_string(),
            num_layers_str: "1".to_string(),
            layers: Vec::new(),
            mesh_length_nm: "0.1".to_string(),
            energy_step_mev: "0.1".to_string(),
            barrier_height_ev: String::new(),
            ec_ef_bottom_ev: String::new(),
        }
    }

    fn field_count(&self) -> usize {
        match &self.page {
            Page::SimSettings => 5,
            Page::Measurement => 9,
            Page::StructureCount => 1,
            Page::Layer(i) => self.layers.get(*i).map(|l| l.field_count()).unwrap_or(0),
            Page::MeshBoundary => 4,
            Page::Confirm => 0,
        }
    }

    fn next_field(&mut self) {
        let n = self.field_count();
        if n > 0 {
            self.focused = (self.focused + 1) % n;
            self.error = None;
        }
    }

    fn prev_field(&mut self) {
        let n = self.field_count();
        if n > 0 {
            self.focused = (self.focused + n - 1) % n;
            self.error = None;
        }
    }

    fn is_toggle(&self) -> bool {
        matches!((&self.page, self.focused), (Page::SimSettings, 4) | (Page::Layer(_), 1))
    }

    fn toggle_focused(&mut self) {
        match (&self.page, self.focused) {
            (Page::SimSettings, 4) => self.parallel = !self.parallel,
            (Page::Layer(i), 1) => {
                let i = *i;
                if let Some(layer) = self.layers.get_mut(i) {
                    layer.material = match layer.material {
                        MaterialType::Semiconductor => MaterialType::Insulator,
                        MaterialType::Insulator => MaterialType::Semiconductor,
                    };
                    // Clamp focused index when fields shrink (SC→Ins removes 3 fields)
                    let fc = layer.field_count();
                    self.focused = self.focused.min(fc - 1);
                }
            }
            _ => {}
        }
    }

    fn active_text_field(&mut self) -> Option<&mut String> {
        match self.page.clone() {
            Page::SimSettings => match self.focused {
                0 => Some(&mut self.sim_name),
                1 => Some(&mut self.sor_factor),
                2 => Some(&mut self.convergence),
                3 => Some(&mut self.max_iter),
                _ => None,
            },
            Page::Measurement => match self.focused {
                0 => Some(&mut self.temperature),
                1 => Some(&mut self.v_start),
                2 => Some(&mut self.v_end),
                3 => Some(&mut self.v_step),
                4 => Some(&mut self.ac_voltage),
                5 => Some(&mut self.meas_time),
                6 => Some(&mut self.stress_voltage),
                7 => Some(&mut self.stress_relief_voltage),
                8 => Some(&mut self.stress_relief_time),
                _ => None,
            },
            Page::StructureCount => Some(&mut self.num_layers_str),
            Page::Layer(i) => {
                let f = self.focused;
                let layer = self.layers.get_mut(i)?;
                match f {
                    0 => Some(&mut layer.name),
                    1 => None,
                    2 => Some(&mut layer.thickness_nm),
                    3 => Some(&mut layer.permittivity),
                    4 => Some(&mut layer.bandgap_ev),
                    5 => Some(&mut layer.delta_cb_ev),
                    6 => Some(&mut layer.mass_electron_coeff),
                    7 => Some(&mut layer.donor_conc_cm3),
                    8 => Some(&mut layer.energy_donor_ev),
                    _ => None,
                }
            }
            Page::MeshBoundary => match self.focused {
                0 => Some(&mut self.mesh_length_nm),
                1 => Some(&mut self.energy_step_mev),
                2 => Some(&mut self.barrier_height_ev),
                3 => Some(&mut self.ec_ef_bottom_ev),
                _ => None,
            },
            Page::Confirm => None,
        }
    }

    fn type_char(&mut self, c: char) {
        if let Some(f) = self.active_text_field() {
            f.push(c);
        }
    }

    fn backspace(&mut self) {
        if let Some(f) = self.active_text_field() {
            f.pop();
        }
    }

    // ─── Validation helpers ───────────────────────────────────────────────────

    fn validate_sim_settings(&self) -> Result<(), String> {
        let name = self.sim_name.trim();
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        {
            return Err(
                "Name: letters/digits/'-'/'_'/'.' only, cannot be empty or contain '..'".into(),
            );
        }
        if name.contains("..") {
            return Err("Name cannot contain '..'".into());
        }
        self.sor_factor
            .trim()
            .parse::<f64>()
            .map_err(|_| "SOR factor must be a number (e.g. 1.9)".to_string())?;
        self.convergence
            .trim()
            .parse::<f64>()
            .map_err(|_| "Convergence must be a number (e.g. 1e-6)".to_string())?;
        self.max_iter
            .trim()
            .parse::<usize>()
            .map_err(|_| "Max iterations must be a positive integer".to_string())?;
        Ok(())
    }

    fn validate_measurement(&self) -> Result<(), String> {
        macro_rules! parse_f64 {
            ($v:expr, $label:literal) => {
                $v.trim()
                    .parse::<f64>()
                    .map_err(|_| format!("{} must be a number", $label))?
            };
        }
        let temp = parse_f64!(self.temperature, "Temperature");
        if temp <= 0.0 {
            return Err("Temperature must be > 0".into());
        }
        parse_f64!(self.v_start, "Voltage start");
        parse_f64!(self.v_end, "Voltage end");
        let step = parse_f64!(self.v_step, "Voltage step");
        if step == 0.0 {
            return Err("Voltage step cannot be zero".into());
        }
        parse_f64!(self.ac_voltage, "AC voltage");
        parse_f64!(self.meas_time, "Measurement time");
        parse_f64!(self.stress_voltage, "Stress voltage");
        parse_f64!(self.stress_relief_voltage, "Stress relief voltage");
        parse_f64!(self.stress_relief_time, "Stress relief time");
        Ok(())
    }

    fn validate_layer(&self, i: usize) -> Result<(), String> {
        let layer = &self.layers[i];
        macro_rules! parse_pos_f64 {
            ($v:expr, $label:literal) => {{
                let v: f64 = $v
                    .trim()
                    .parse()
                    .map_err(|_| format!("{} must be a positive number", $label))?;
                if v <= 0.0 {
                    return Err(format!("{} must be > 0", $label));
                }
                v
            }};
        }
        parse_pos_f64!(layer.thickness_nm, "Thickness");
        parse_pos_f64!(layer.permittivity, "Permittivity");
        parse_pos_f64!(layer.bandgap_ev, "Bandgap");
        layer
            .delta_cb_ev
            .trim()
            .parse::<f64>()
            .map_err(|_| "Delta Ec must be a number".to_string())?;
        if layer.is_semiconductor() {
            parse_pos_f64!(layer.mass_electron_coeff, "Effective mass");
            parse_pos_f64!(layer.donor_conc_cm3, "Donor concentration");
            layer
                .energy_donor_ev
                .trim()
                .parse::<f64>()
                .map_err(|_| "Energy level donor must be a number".to_string())?;
        }
        Ok(())
    }

    fn validate_mesh_boundary(&self) -> Result<(), String> {
        let len: f64 = self
            .mesh_length_nm
            .trim()
            .parse()
            .map_err(|_| "Mesh length must be a positive number".to_string())?;
        if len <= 0.0 {
            return Err("Mesh length must be > 0".into());
        }
        self.energy_step_mev
            .trim()
            .parse::<f64>()
            .map_err(|_| "Energy step must be a number".to_string())?;
        self.barrier_height_ev
            .trim()
            .parse::<f64>()
            .map_err(|_| "Barrier height must be a number".to_string())?;
        self.ec_ef_bottom_ev
            .trim()
            .parse::<f64>()
            .map_err(|_| "Ec-Ef bottom must be a number".to_string())?;
        Ok(())
    }

    // ─── Navigation ───────────────────────────────────────────────────────────

    fn validate_and_advance(&mut self) {
        self.error = None;
        let result = match self.page.clone() {
            Page::SimSettings => self.validate_sim_settings().map(|_| Page::Measurement),
            Page::Measurement => self.validate_measurement().map(|_| Page::StructureCount),
            Page::StructureCount => {
                match self.num_layers_str.trim().parse::<usize>() {
                    Ok(n) if n > 0 => {
                        // Grow or shrink layer vec as needed
                        while self.layers.len() < n {
                            let idx = self.layers.len();
                            self.layers.push(LayerInput::new(idx));
                        }
                        self.layers.truncate(n);
                        Ok(Page::Layer(0))
                    }
                    _ => Err("Number of layers must be ≥ 1".to_string()),
                }
            }
            Page::Layer(i) => self.validate_layer(i).map(|_| {
                let n = self.layers.len();
                if i + 1 < n {
                    Page::Layer(i + 1)
                } else {
                    Page::MeshBoundary
                }
            }),
            Page::MeshBoundary => self.validate_mesh_boundary().map(|_| Page::Confirm),
            Page::Confirm => return, // handled by caller
        };

        match result {
            Ok(next) => {
                self.page = next;
                self.focused = 0;
            }
            Err(msg) => self.error = Some(msg),
        }
    }

    fn go_back(&mut self) {
        self.error = None;
        self.focused = 0;
        self.page = match self.page.clone() {
            Page::SimSettings => Page::SimSettings,
            Page::Measurement => Page::SimSettings,
            Page::StructureCount => Page::Measurement,
            Page::Layer(0) => Page::StructureCount,
            Page::Layer(i) => Page::Layer(i - 1),
            Page::MeshBoundary => {
                let n = self.layers.len();
                if n > 0 {
                    Page::Layer(n - 1)
                } else {
                    Page::StructureCount
                }
            }
            Page::Confirm => Page::MeshBoundary,
        };
    }

    // ─── Build ────────────────────────────────────────────────────────────────

    fn build_config(self) -> ConfigurationBuilder {
        let sim_settings = SimSettings {
            sim_name: self.sim_name.trim().to_string(),
            sor_relaxation_factor: self.sor_factor.trim().parse().unwrap(),
            convergence_criterion: self.convergence.trim().parse().unwrap(),
            max_iterations: self.max_iter.trim().parse().unwrap(),
            parallel_use: self.parallel,
        };

        let measurement = Measurement {
            temperature: Temperature {
                temperature: self.temperature.trim().parse().unwrap(),
            },
            voltage: Voltage {
                start: self.v_start.trim().parse().unwrap(),
                end: self.v_end.trim().parse().unwrap(),
                step: self.v_step.trim().parse().unwrap(),
            },
            ac_voltage: self.ac_voltage.trim().parse::<f64>().unwrap() * MV_TO_V,
            time: Time {
                measurement_time: self.meas_time.trim().parse().unwrap(),
            },
            stress: Stress {
                stress_voltage: self.stress_voltage.trim().parse().unwrap(),
                stress_relief_voltage: self.stress_relief_voltage.trim().parse().unwrap(),
                stress_relief_time: self.stress_relief_time.trim().parse().unwrap(),
            },
        };

        let n = self.layers.len();
        let mut device_structure = DeviceStructure {
            id: (0..n as u32).collect(),
            name: Vec::new(),
            material_type: Vec::new(),
            thickness: Vec::new(),
            mass_electron: Vec::new(),
            permittivity: Vec::new(),
            bandgap_energy: Vec::new(),
            delta_conduction_band: Vec::new(),
            donor_concentration: Vec::new(),
            energy_level_donor: Vec::new(),
        };

        for (i, layer) in self.layers.iter().enumerate() {
            device_structure.name.push(layer.name.trim().to_string());
            device_structure.material_type.push(layer.material);
            device_structure
                .thickness
                .push(layer.thickness_nm.trim().parse::<f64>().unwrap() * NM_TO_M);
            device_structure
                .permittivity
                .push(layer.permittivity.trim().parse::<f64>().unwrap() * EPSILON_0);
            device_structure
                .bandgap_energy
                .push(layer.bandgap_ev.trim().parse().unwrap());
            // Last layer delta_cb is always 0
            let dcb = if i == n - 1 {
                0.0
            } else {
                layer.delta_cb_ev.trim().parse().unwrap()
            };
            device_structure.delta_conduction_band.push(dcb);
            if layer.is_semiconductor() {
                device_structure.mass_electron.push(
                    layer.mass_electron_coeff.trim().parse::<f64>().unwrap() * M_ELECTRON,
                );
                device_structure.donor_concentration.push(
                    layer.donor_conc_cm3.trim().parse::<f64>().unwrap() * PER_CM3_TO_PER_M3,
                );
                device_structure
                    .energy_level_donor
                    .push(layer.energy_donor_ev.trim().parse().unwrap());
            } else {
                device_structure.mass_electron.push(0.0);
                device_structure.donor_concentration.push(0.0);
                device_structure.energy_level_donor.push(0.0);
            }
        }

        let bulk_fixed_charge = BulkFixedCharge {
            layer_id: (0..n as u32).collect(),
            charge_density: vec![0.0; n],
        };
        let interface_fixed_charge = InterfaceFixedCharge {
            interface_id: (0..n.saturating_sub(1) as u32).collect(),
            charge_density: vec![0.0; n.saturating_sub(1)],
        };
        let continuous_interface_states = ContinuousInterfaceStatesConfig {
            interface_id: vec![],
            parameters: vec![],
        };
        let discrete_interface_states = DiscreteInterfaceStatesConfig {
            interface_id: vec![],
            parameters: vec![],
        };
        let capture_cross_section = CaptureCrossSectionConfig {
            interface_id: vec![],
            model: vec![],
            mass_electron: vec![],
        };

        let total_thickness: f64 = device_structure.thickness.iter().sum();
        let mesh_params = MeshParams {
            layer_id: vec![0],
            length_per_layer: vec![
                self.mesh_length_nm.trim().parse::<f64>().unwrap() * NM_TO_M,
            ],
            layer_thickness: vec![total_thickness],
            energy_step: self.energy_step_mev.trim().parse::<f64>().unwrap() * MEV_TO_EV,
        };

        let boundary_conditions = BoundaryConditions {
            barrier_height: self.barrier_height_ev.trim().parse().unwrap(),
            ec_ef_bottom: self.ec_ef_bottom_ev.trim().parse().unwrap(),
        };

        ConfigurationBuilder::new(Configuration {
            sim_settings,
            measurement,
            device_structure,
            bulk_fixed_charge,
            interface_fixed_charge,
            continuous_interface_states,
            discrete_interface_states,
            capture_cross_section,
            mesh_params,
            boundary_conditions,
        })
    }
}

// ─── Rendering ────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, app: &App) {
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
        Page::SimSettings => " [1/5] Simulation Settings ".to_string(),
        Page::Measurement => " [2/5] Measurement ".to_string(),
        Page::StructureCount => " [3/5] Device Structure ".to_string(),
        Page::Layer(i) => format!(" [3/5] Layer {} of {} ", i + 1, app.layers.len()),
        Page::MeshBoundary => " [4/5] Mesh & Boundary Conditions ".to_string(),
        Page::Confirm => " [5/5] Confirm & Run ".to_string(),
    };
    let header = Paragraph::new("  GaN C-V Simulator")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(header, area);
}

fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    match &app.page {
        Page::SimSettings => draw_sim_settings(frame, area, app),
        Page::Measurement => draw_measurement(frame, area, app),
        Page::StructureCount => draw_structure_count(frame, area, app),
        Page::Layer(i) => draw_layer(frame, area, app, *i),
        Page::MeshBoundary => draw_mesh_boundary(frame, area, app),
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

fn draw_sim_settings(frame: &mut Frame, area: Rect, app: &App) {
    let fields = [
        ("Simulation Name", app.sim_name.clone(), false),
        ("SOR Relaxation Factor", app.sor_factor.clone(), false),
        ("Convergence Criterion (eV)", app.convergence.clone(), false),
        ("Max Iterations", app.max_iter.clone(), false),
        (
            "Parallel Processing",
            if app.parallel { "ON".to_string() } else { "OFF".to_string() },
            true,
        ),
    ];
    let lines = field_lines(&fields, app.focused);
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Fields "));
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
        ("Stress Relief Voltage (V)", app.stress_relief_voltage.clone(), false),
        ("Stress Relief Time (s)", app.stress_relief_time.clone(), false),
    ];
    let lines = field_lines(&fields, app.focused);
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Fields "));
    frame.render_widget(para, area);
}

fn draw_structure_count(frame: &mut Frame, area: Rect, app: &App) {
    let fields = [("Number of Layers", app.num_layers_str.clone(), false)];
    let lines = field_lines(&fields, app.focused);
    let note = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Fields "));
    frame.render_widget(note, area);
}

fn draw_layer(frame: &mut Frame, area: Rect, app: &App, i: usize) {
    let layer = &app.layers[i];
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
        ("Delta Conduction Band (eV)", layer.delta_cb_ev.clone(), false),
    ];
    if layer.is_semiconductor() {
        fields.push(("Effective Mass Coeff", layer.mass_electron_coeff.clone(), false));
        fields.push(("Donor Concentration (cm^-3)", layer.donor_conc_cm3.clone(), false));
        fields.push(("Energy Level Donor Ec-Ed (eV)", layer.energy_donor_ev.clone(), false));
    }
    let lines = field_lines(&fields, app.focused);
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(format!(" Layer {i} ")));
    frame.render_widget(para, area);
}

fn draw_mesh_boundary(frame: &mut Frame, area: Rect, app: &App) {
    let fields = [
        ("Mesh Length (nm)", app.mesh_length_nm.clone(), false),
        ("Energy Step (meV)", app.energy_step_mev.clone(), false),
        ("Barrier Height (eV)", app.barrier_height_ev.clone(), false),
        ("Ec - Ef Bottom (eV)", app.ec_ef_bottom_ev.clone(), false),
    ];
    let note_text = "\n  Note: A single uniform mesh layer is used for the full device.\n  Fixed charges and interface states default to zero/none.";
    let lines = field_lines(&fields, app.focused);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(4)])
        .split(area);
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Fields "));
    frame.render_widget(para, chunks[0]);
    let note = Paragraph::new(note_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Info "));
    frame.render_widget(note, chunks[1]);
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
    let summary = format!(
        "\n  Simulation:   {}\n  Temperature:  {} K\n  Voltage:      {} → {} V  (step {} V)\n  AC Voltage:   {} mV\n\n{}  Mesh length:  {} nm\n  Energy step:  {} meV\n  Barrier:      {} eV\n  Ec-Ef bottom: {} eV\n\n  Fixed charges and interface states: all zero / none (default)\n\n  Press Enter to start simulation.",
        app.sim_name,
        app.temperature,
        app.v_start,
        app.v_end,
        app.v_step,
        app.ac_voltage,
        layer_summary,
        app.mesh_length_nm,
        app.energy_step_mev,
        app.barrier_height_ev,
        app.ec_ef_bottom_ev,
    );
    let para = Paragraph::new(summary)
        .block(Block::default().borders(Borders::ALL).title(" Summary "))
        .style(Style::default().fg(Color::Green));
    frame.render_widget(para, area);
}

// ─── Event loop ───────────────────────────────────────────────────────────────

/// Run the TUI configuration wizard and return a [`ConfigurationBuilder`].
pub fn run_tui() -> anyhow::Result<ConfigurationBuilder> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal);

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> anyhow::Result<ConfigurationBuilder> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| draw(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Err(anyhow::anyhow!("TUI cancelled by user"));
                }
                KeyCode::Char(' ') => {
                    if app.is_toggle() {
                        app.toggle_focused();
                    }
                    // Space is ignored on text fields (not valid in names/numbers)
                }
                KeyCode::Char(c) => app.type_char(c),
                KeyCode::Backspace => app.backspace(),
                KeyCode::Tab | KeyCode::Down => app.next_field(),
                KeyCode::BackTab | KeyCode::Up => app.prev_field(),
                KeyCode::Enter | KeyCode::Right => {
                    if app.page == Page::Confirm {
                        return Ok(app.build_config());
                    }
                    app.validate_and_advance();
                }
                KeyCode::Esc | KeyCode::Left => app.go_back(),
                _ => {}
            }
        }
    }
}
