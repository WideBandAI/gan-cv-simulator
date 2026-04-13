pub mod forms;
mod preview;

use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::config::boundary_conditions::BoundaryConditions;
use crate::config::capture_cross_section::CaptureCrossSectionConfig;
use crate::config::configuration_builder::{Configuration, ConfigurationBuilder};
use crate::config::fixcharge::{BulkFixedCharge, InterfaceFixedCharge};
use crate::config::interface_states::{ContinuousInterfaceStatesConfig, DiscreteInterfaceStatesConfig};
use crate::config::mesh::MeshParams;
use crate::config::structure::MaterialType;
use crate::constants::units::{MEV_TO_EV, NM_TO_M};
use crate::physics_equations::equilibrium_potential::equilibrium_potential_n_type;
use crate::tui::forms::{
    Field,
    measurement::MeasurementForm,
    sim_settings::SimSettingsForm,
    structure::StructureForm,
};

// ── Tab ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Settings = 0,
    Structure = 1,
    Measurement = 2,
    Run = 3,
}

impl Tab {

}

// ── App ───────────────────────────────────────────────────────────────────────

pub(crate) struct App {
    tab: Tab,
    pub(crate) sim_form: SimSettingsForm,
    pub(crate) struct_form: StructureForm,
    pub(crate) measure_form: MeasurementForm,
    pub(crate) barrier_field: Field,
    errors: Vec<String>,
    quit: bool,
    done: bool,
}

impl App {
    fn new() -> Self {
        Self {
            tab: Tab::Settings,
            sim_form: SimSettingsForm::default(),
            struct_form: StructureForm::default(),
            measure_form: MeasurementForm::default(),
            barrier_field: Field::text("Barrier height (eV)", "1.6"),
            errors: vec![],
            quit: false,
            done: false,
        }
    }

    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // Global: Ctrl+C quits
        if key == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            self.quit = true;
            return;
        }

        // Tab switching: F1–F4
        match key {
            KeyCode::F(1) => { self.tab = Tab::Settings; return; }
            KeyCode::F(2) => { self.tab = Tab::Structure; return; }
            KeyCode::F(3) => { self.tab = Tab::Measurement; return; }
            KeyCode::F(4) => { self.tab = Tab::Run; return; }
            _ => {}
        }

        // Per-tab handling
        match self.tab {
            Tab::Settings => { self.sim_form.handle_key(key); }
            Tab::Structure => { self.struct_form.handle_key(key); }
            Tab::Measurement => { self.measure_form.handle_key(key); }
            Tab::Run => match key {
                KeyCode::Char('q') => { self.quit = true; }
                KeyCode::Char('r') | KeyCode::Enter => {
                    self.errors.clear();
                    self.try_run();
                }
                _ => { self.barrier_field.handle_key(key); }
            },
        }
    }

    fn try_run(&mut self) {
        let mut errors = Vec::new();
        if let Err(e) = self.sim_form.build() { errors.extend(e); }
        if let Err(e) = self.measure_form.build() { errors.extend(e); }
        if let Err(e) = self.struct_form.build() { errors.extend(e); }
        if self.barrier_field.text_value().trim().parse::<f64>().is_err() {
            errors.push("Barrier height: invalid number".into());
        }
        if errors.is_empty() {
            self.done = true;
        } else {
            self.errors = errors;
        }
    }

    fn build_config(&mut self) -> anyhow::Result<ConfigurationBuilder> {
        let sim_settings = self
            .sim_form
            .build()
            .map_err(|e: Vec<String>| anyhow::anyhow!("{}", e.join(", ")))?;
        let measurement = self
            .measure_form
            .build()
            .map_err(|e: Vec<String>| anyhow::anyhow!("{}", e.join(", ")))?;
        let device_structure = self
            .struct_form
            .build()
            .map_err(|e: Vec<String>| anyhow::anyhow!("{}", e.join(", ")))?;

        let barrier_height = self
            .barrier_field
            .text_value()
            .trim()
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("Barrier height: invalid number"))?;

        // Auto-compute Ec–Ef from equilibrium (bottom semiconductor layer)
        let ec_ef_bottom = match (
            device_structure.material_type.last(),
            device_structure.mass_electron.last(),
            device_structure.donor_concentration.last(),
        ) {
            (Some(&MaterialType::Semiconductor), Some(&me), Some(&nd)) => {
                equilibrium_potential_n_type(me, nd, measurement.temperature.temperature)
            }
            _ => 0.0,
        };

        let boundary_conditions = BoundaryConditions { barrier_height, ec_ef_bottom };

        // Default mesh: single region, 0.1 nm step, full device thickness
        let total_thickness: f64 = device_structure.thickness.iter().sum();
        let mesh_params = MeshParams {
            layer_id: vec![0],
            length_per_layer: vec![0.1 * NM_TO_M],
            layer_thickness: vec![total_thickness],
            energy_step: 0.1 * MEV_TO_EV,
        };

        // Zero fixed charges (most common case; advanced users use JSON)
        let bulk_fixed_charge = BulkFixedCharge { layer_id: vec![], charge_density: vec![] };
        let interface_fixed_charge =
            InterfaceFixedCharge { interface_id: vec![], charge_density: vec![] };

        // No interface states
        let continuous_interface_states =
            ContinuousInterfaceStatesConfig { interface_id: vec![], parameters: vec![] };
        let discrete_interface_states =
            DiscreteInterfaceStatesConfig { interface_id: vec![], parameters: vec![] };
        let capture_cross_section =
            CaptureCrossSectionConfig { interface_id: vec![], model: vec![], mass_electron: vec![] };

        Ok(ConfigurationBuilder::new(Configuration {
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
        }))
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [tab_area, main_area, status_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        // ── Tab bar ────────────────────────────────────────────────────────
        let titles = vec!["[F1] Settings", "[F2] Structure", "[F3] Measurement", "[F4] Run"];
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" GaN C-V Simulator "),
            )
            .select(self.tab as usize)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            );
        frame.render_widget(tabs, tab_area);

        // ── Split: left form | right preview ──────────────────────────────
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(main_area);

        match self.tab {
            Tab::Settings => {
                self.sim_form.render(frame, left_area);
                preview::render_settings_preview(frame, &self.sim_form, right_area);
            }
            Tab::Structure => {
                self.struct_form.render(frame, left_area);
                preview::render_structure_preview(frame, &self.struct_form, right_area);
            }
            Tab::Measurement => {
                self.measure_form.render(frame, left_area);
                preview::render_measurement_preview(frame, &self.measure_form, right_area);
            }
            Tab::Run => {
                preview::render_run_summary(frame, self, left_area);
                preview::render_run_preview(frame, &self.barrier_field, &self.errors, right_area);
            }
        }

        // ── Status bar ────────────────────────────────────────────────────
        let hint = match self.tab {
            Tab::Settings => "Tab/\u{2191}\u{2193}: navigate field  F1\u{2013}F4: switch tab  Ctrl+C: quit",
            Tab::Structure => "\u{2191}\u{2193}: select layer  Enter: edit  Esc: back  a: add  d: delete  F1\u{2013}F4: switch tab",
            Tab::Measurement => "Tab/\u{2191}\u{2193}: navigate field  F1\u{2013}F4: switch tab  Ctrl+C: quit",
            Tab::Run => "r/Enter: run simulation  q: quit  F1\u{2013}F4: switch tab",
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                hint,
                Style::default().fg(Color::DarkGray),
            ))),
            status_area,
        );
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Launch the interactive TUI and return a `ConfigurationBuilder` on success.
pub fn run() -> anyhow::Result<ConfigurationBuilder> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let result: anyhow::Result<ConfigurationBuilder> = (|| loop {
        terminal.draw(|f| app.draw(f))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers);
                }
            }
        }

        if app.quit {
            return Err(anyhow::anyhow!("cancelled"));
        }
        if app.done {
            return app.build_config();
        }
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
