use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::io;

use crate::config::{
    boundary_conditions::BoundaryConditions,
    capture_cross_section::{CaptureCrossSectionConfig, CaptureCrossSectionModel},
    configuration_builder::{Configuration, ConfigurationBuilder},
    fixcharge::{BulkFixedCharge, InterfaceFixedCharge},
    interface_states::{ContinuousInterfaceStatesConfig, DiscreteInterfaceStatesConfig},
    measurement::{Measurement, Stress, Temperature, Time, Voltage},
    mesh::MeshParams,
    sim_settings::SimSettings,
    structure::{DeviceStructure, MaterialType},
};
use crate::constants::physics::{EPSILON_0, M_ELECTRON};
use crate::constants::units::{
    CM2_TO_M2, MEV_TO_EV, MV_TO_V, NM_TO_M, PER_CM2_TO_PER_M2, PER_CM3_TO_PER_M3,
};
use crate::physics_equations::equilibrium_potential::equilibrium_potential_n_type;
use crate::physics_equations::interface_states::{DIGSModel, DiscreteModel, DiscreteStateType};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum EcEfMode {
    Manual,
    Equilibrium,
}

// ─── Interface states input structs ──────────────────────────────────────────

#[derive(Debug, Clone)]
struct ContinuousStateInput {
    dit0: String,
    nssec: String,
    nssev: String,
    ecnl: String,
    nd: String,
    na: String,
}

impl ContinuousStateInput {
    fn new() -> Self {
        Self {
            dit0: "1e12".to_string(),
            nssec: "10".to_string(),
            nssev: "10".to_string(),
            ecnl: "1.3".to_string(),
            nd: "3".to_string(),
            na: "3".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct DiscreteStateInput {
    ditmax: String,
    ed: String,
    fwhm: String,
    state_type: DiscreteStateType,
}

impl DiscreteStateInput {
    fn new() -> Self {
        Self {
            ditmax: "1e12".to_string(),
            ed: "0.5".to_string(),
            fwhm: "0.3".to_string(),
            state_type: DiscreteStateType::DonorLike,
        }
    }
}

#[derive(Debug, Clone)]
struct InterfaceStateInput {
    has_continuous: bool,
    continuous: ContinuousStateInput,
    has_discrete: bool,
    num_discrete_str: String,
    discrete_traps: Vec<DiscreteStateInput>,
}

impl InterfaceStateInput {
    fn new() -> Self {
        Self {
            has_continuous: false,
            continuous: ContinuousStateInput::new(),
            has_discrete: false,
            num_discrete_str: "1".to_string(),
            discrete_traps: Vec::new(),
        }
    }
}

// ─── Capture cross section input structs ─────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum CsModelType {
    Constant,
    EnergyDependent,
}

#[derive(Debug, Clone)]
struct CaptureCrossSectionInput {
    model_type: CsModelType,
    sigma: String,
    sigma_mid: String,
    e_mid: String,
    e_slope: String,
    mass_electron_coeff: String,
}

impl CaptureCrossSectionInput {
    fn new_with_default_mass(mass_coeff: f64) -> Self {
        Self {
            model_type: CsModelType::Constant,
            sigma: "1e-16".to_string(),
            sigma_mid: "1e-16".to_string(),
            e_mid: "0.5".to_string(),
            e_slope: "0.1".to_string(),
            mass_electron_coeff: format!("{:.4}", mass_coeff),
        }
    }

    fn field_count(&self) -> usize {
        match self.model_type {
            CsModelType::Constant => 3,
            CsModelType::EnergyDependent => 5,
        }
    }
}

// ─── Pages ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Page {
    SimSettings,
    Measurement,
    StructureCount,
    Layer(usize),
    MeshCount,
    MeshLayer(usize),
    FixedCharge,
    InterfaceStates(usize),
    DiscreteState(usize, usize),
    CaptureCrossSection(usize),
    BoundaryConditions,
    Confirm,
}

// ─── Helper ───────────────────────────────────────────────────────────────────

/// Returns the indices (into interface_states) of interfaces that have any states defined.
fn active_interface_indices(interface_states: &[InterfaceStateInput]) -> Vec<usize> {
    interface_states
        .iter()
        .enumerate()
        .filter(|(_, ist)| ist.has_continuous || ist.has_discrete)
        .map(|(i, _)| i)
        .collect()
}

// ─── Input state structs ──────────────────────────────────────────────────────

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

    fn field_count(&self, is_last: bool) -> usize {
        let base = if self.is_semiconductor() { 9 } else { 6 };
        if is_last { base - 1 } else { base }
    }
}

#[derive(Debug, Clone)]
struct MeshLayerInput {
    mesh_length_nm: String,
    thickness_nm: String,
}

impl MeshLayerInput {
    fn new() -> Self {
        Self {
            mesh_length_nm: "0.1".to_string(),
            thickness_nm: String::new(),
        }
    }

    /// Non-last layer: 2 fields (mesh_length, thickness).
    /// Last layer: 1 field (mesh_length only; thickness is auto-calculated).
    fn field_count(is_last: bool) -> usize {
        if is_last { 1 } else { 2 }
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

    // Device structure
    num_layers_str: String,
    layers: Vec<LayerInput>,

    // Mesh
    num_mesh_layers_str: String,
    energy_step_mev: String,
    mesh_layers: Vec<MeshLayerInput>,

    // Fixed charges
    bulk_charge_densities: Vec<String>,      // C/cm^3 per layer
    interface_charge_densities: Vec<String>, // C/cm^2 per interface

    // Interface states
    interface_states: Vec<InterfaceStateInput>,

    // Capture cross section (one per active interface, in same order as active_interface_indices)
    capture_cross_sections: Vec<CaptureCrossSectionInput>,

    // Boundary conditions
    barrier_height_ev: String,
    ec_ef_mode: EcEfMode,
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
            num_mesh_layers_str: "1".to_string(),
            energy_step_mev: "0.1".to_string(),
            mesh_layers: Vec::new(),
            bulk_charge_densities: Vec::new(),
            interface_charge_densities: Vec::new(),
            interface_states: Vec::new(),
            capture_cross_sections: Vec::new(),
            barrier_height_ev: String::new(),
            ec_ef_mode: EcEfMode::Manual,
            ec_ef_bottom_ev: String::new(),
        }
    }

    // ─── Field counts ─────────────────────────────────────────────────────────

    fn field_count(&self) -> usize {
        match &self.page {
            Page::SimSettings => 5,
            Page::Measurement => 9,
            Page::StructureCount => 1,
            Page::Layer(i) => {
                let is_last = *i + 1 == self.layers.len();
                self.layers
                    .get(*i)
                    .map(|l| l.field_count(is_last))
                    .unwrap_or(0)
            }
            Page::MeshCount => 2,
            Page::MeshLayer(i) => {
                let is_last = *i + 1 == self.mesh_layers.len();
                MeshLayerInput::field_count(is_last)
            }
            Page::FixedCharge => {
                let n = self.layers.len();
                n + n.saturating_sub(1) // n bulk + (n-1) interface
            }
            Page::InterfaceStates(i) => {
                if let Some(ist) = self.interface_states.get(*i) {
                    let cont_fields = if ist.has_continuous { 6 } else { 0 };
                    let disc_fields = if ist.has_discrete { 1 } else { 0 };
                    1 + cont_fields + 1 + disc_fields
                } else {
                    2
                }
            }
            Page::DiscreteState(_, _) => 4,
            Page::CaptureCrossSection(k) => self
                .capture_cross_sections
                .get(*k)
                .map(|c| c.field_count())
                .unwrap_or(0),
            Page::BoundaryConditions => {
                let bottom_is_sc = self
                    .layers
                    .last()
                    .map(|l| l.is_semiconductor())
                    .unwrap_or(false);
                if bottom_is_sc {
                    if self.ec_ef_mode == EcEfMode::Manual {
                        3
                    } else {
                        2
                    }
                } else {
                    2
                }
            }
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

    // ─── Toggle ───────────────────────────────────────────────────────────────

    fn is_toggle(&self) -> bool {
        match (&self.page, self.focused) {
            (Page::SimSettings, 4) | (Page::Layer(_), 1) => true,
            (Page::BoundaryConditions, 1) => self
                .layers
                .last()
                .map(|l| l.is_semiconductor())
                .unwrap_or(false),
            (Page::InterfaceStates(i), f) => {
                if let Some(ist) = self.interface_states.get(*i) {
                    let cont_fields = if ist.has_continuous { 6 } else { 0 };
                    let disc_toggle_idx = 1 + cont_fields;
                    f == 0 || f == disc_toggle_idx
                } else {
                    false
                }
            }
            (Page::DiscreteState(_, _), 3) => true,
            (Page::CaptureCrossSection(_), 0) => true,
            _ => false,
        }
    }

    fn toggle_focused(&mut self) {
        match (&self.page, self.focused) {
            (Page::SimSettings, 4) => self.parallel = !self.parallel,
            (Page::Layer(i), 1) => {
                let i = *i;
                let is_last = i + 1 == self.layers.len();
                if let Some(layer) = self.layers.get_mut(i) {
                    layer.material = match layer.material {
                        MaterialType::Semiconductor => MaterialType::Insulator,
                        MaterialType::Insulator => MaterialType::Semiconductor,
                    };
                    let fc = layer.field_count(is_last);
                    self.focused = self.focused.min(fc - 1);
                }
            }
            (Page::BoundaryConditions, 1) => {
                let bottom_is_sc = self
                    .layers
                    .last()
                    .map(|l| l.is_semiconductor())
                    .unwrap_or(false);
                if bottom_is_sc {
                    self.ec_ef_mode = match self.ec_ef_mode {
                        EcEfMode::Manual => EcEfMode::Equilibrium,
                        EcEfMode::Equilibrium => EcEfMode::Manual,
                    };
                    // Clamp: if switching to Equilibrium, field 2 disappears
                    let fc = self.field_count();
                    self.focused = self.focused.min(fc - 1);
                }
            }
            (Page::InterfaceStates(i), f) => {
                let i = *i;
                let cont_fields = self
                    .interface_states
                    .get(i)
                    .map(|ist| if ist.has_continuous { 6 } else { 0 })
                    .unwrap_or(0);
                let disc_toggle_idx = 1 + cont_fields;
                if f == 0 {
                    if let Some(ist) = self.interface_states.get_mut(i) {
                        ist.has_continuous = !ist.has_continuous;
                    }
                    let fc = self.field_count();
                    self.focused = self.focused.min(fc.saturating_sub(1));
                } else if f == disc_toggle_idx {
                    if let Some(ist) = self.interface_states.get_mut(i) {
                        ist.has_discrete = !ist.has_discrete;
                    }
                    let fc = self.field_count();
                    self.focused = self.focused.min(fc.saturating_sub(1));
                }
            }
            (Page::CaptureCrossSection(k), 0) => {
                let k = *k;
                if let Some(ccs) = self.capture_cross_sections.get_mut(k) {
                    ccs.model_type = match ccs.model_type {
                        CsModelType::Constant => CsModelType::EnergyDependent,
                        CsModelType::EnergyDependent => CsModelType::Constant,
                    };
                    let fc = ccs.field_count();
                    self.focused = self.focused.min(fc - 1);
                }
            }
            (Page::DiscreteState(i, j), 3) => {
                let (i, j) = (*i, *j);
                if let Some(trap) = self
                    .interface_states
                    .get_mut(i)
                    .and_then(|ist| ist.discrete_traps.get_mut(j))
                {
                    trap.state_type = match trap.state_type {
                        DiscreteStateType::DonorLike => DiscreteStateType::AcceptorLike,
                        DiscreteStateType::AcceptorLike => DiscreteStateType::DonorLike,
                    };
                }
            }
            _ => {}
        }
    }

    // ─── Text field access ────────────────────────────────────────────────────

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
                let is_last = i + 1 == self.layers.len();
                let layer = self.layers.get_mut(i)?;
                let idx = if is_last && f >= 5 { f + 1 } else { f };
                match idx {
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
            Page::MeshCount => match self.focused {
                0 => Some(&mut self.num_mesh_layers_str),
                1 => Some(&mut self.energy_step_mev),
                _ => None,
            },
            Page::MeshLayer(i) => {
                let f = self.focused;
                let ml = self.mesh_layers.get_mut(i)?;
                match f {
                    0 => Some(&mut ml.mesh_length_nm),
                    1 => Some(&mut ml.thickness_nm),
                    _ => None,
                }
            }
            Page::FixedCharge => {
                let f = self.focused;
                let n = self.layers.len();
                if f < n {
                    self.bulk_charge_densities.get_mut(f)
                } else {
                    self.interface_charge_densities.get_mut(f - n)
                }
            }
            Page::InterfaceStates(i) => {
                let f = self.focused;
                let cont_fields = self
                    .interface_states
                    .get(i)
                    .map(|ist| if ist.has_continuous { 6 } else { 0 })
                    .unwrap_or(0);
                let disc_toggle_idx = 1 + cont_fields;
                if f == 0 || f == disc_toggle_idx {
                    return None; // toggles
                }
                let ist = self.interface_states.get_mut(i)?;
                if f >= 1 && f < 1 + cont_fields {
                    match f - 1 {
                        0 => Some(&mut ist.continuous.dit0),
                        1 => Some(&mut ist.continuous.nssec),
                        2 => Some(&mut ist.continuous.nssev),
                        3 => Some(&mut ist.continuous.ecnl),
                        4 => Some(&mut ist.continuous.nd),
                        5 => Some(&mut ist.continuous.na),
                        _ => None,
                    }
                } else if f == disc_toggle_idx + 1 && ist.has_discrete {
                    Some(&mut ist.num_discrete_str)
                } else {
                    None
                }
            }
            Page::DiscreteState(i, j) => {
                let f = self.focused;
                if f == 3 {
                    return None; // state_type toggle
                }
                let trap = self
                    .interface_states
                    .get_mut(i)
                    .and_then(|ist| ist.discrete_traps.get_mut(j))?;
                match f {
                    0 => Some(&mut trap.ditmax),
                    1 => Some(&mut trap.ed),
                    2 => Some(&mut trap.fwhm),
                    _ => None,
                }
            }
            Page::CaptureCrossSection(k) => {
                let f = self.focused;
                if f == 0 {
                    return None; // model_type toggle
                }
                let ccs = self.capture_cross_sections.get_mut(k)?;
                match ccs.model_type {
                    CsModelType::Constant => match f {
                        1 => Some(&mut ccs.sigma),
                        2 => Some(&mut ccs.mass_electron_coeff),
                        _ => None,
                    },
                    CsModelType::EnergyDependent => match f {
                        1 => Some(&mut ccs.sigma_mid),
                        2 => Some(&mut ccs.e_mid),
                        3 => Some(&mut ccs.e_slope),
                        4 => Some(&mut ccs.mass_electron_coeff),
                        _ => None,
                    },
                }
            }
            Page::BoundaryConditions => {
                let bottom_is_sc = self
                    .layers
                    .last()
                    .map(|l| l.is_semiconductor())
                    .unwrap_or(false);
                if bottom_is_sc {
                    match self.focused {
                        0 => Some(&mut self.barrier_height_ev),
                        1 => None, // toggle
                        2 => Some(&mut self.ec_ef_bottom_ev),
                        _ => None,
                    }
                } else {
                    match self.focused {
                        0 => Some(&mut self.barrier_height_ev),
                        1 => Some(&mut self.ec_ef_bottom_ev),
                        _ => None,
                    }
                }
            }
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

    // ─── Validation ───────────────────────────────────────────────────────────

    fn validate_sim_settings(&self) -> Result<(), String> {
        let name = self.sim_name.trim();
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        {
            return Err("Name: letters/digits/'-'/'_'/'.' only, cannot be empty".into());
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
        let is_last = i + 1 == self.layers.len();
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
        if !is_last {
            layer
                .delta_cb_ev
                .trim()
                .parse::<f64>()
                .map_err(|_| "Delta Ec must be a number".to_string())?;
        }
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

    fn validate_mesh_count(&self) -> Result<(), String> {
        self.num_mesh_layers_str
            .trim()
            .parse::<usize>()
            .map_err(|_| "Number of mesh layers must be a positive integer".to_string())
            .and_then(|n| {
                if n > 0 {
                    Ok(())
                } else {
                    Err("Number of mesh layers must be ≥ 1".to_string())
                }
            })?;
        self.energy_step_mev
            .trim()
            .parse::<f64>()
            .map_err(|_| "Energy step must be a number (e.g. 0.1)".to_string())?;
        Ok(())
    }

    fn validate_mesh_layer(&self, i: usize) -> Result<(), String> {
        let ml = &self.mesh_layers[i];
        let is_last = i + 1 == self.mesh_layers.len();

        let len: f64 = ml
            .mesh_length_nm
            .trim()
            .parse()
            .map_err(|_| "Mesh length must be a positive number".to_string())?;
        if len <= 0.0 {
            return Err("Mesh length must be > 0".into());
        }

        if !is_last {
            let t: f64 = ml
                .thickness_nm
                .trim()
                .parse()
                .map_err(|_| "Thickness must be a positive number".to_string())?;
            if t <= 0.0 {
                return Err("Thickness must be > 0".into());
            }
            // Check that accumulated thickness doesn't exceed device total
            let total_device_nm: f64 = self
                .layers
                .iter()
                .filter_map(|l| l.thickness_nm.trim().parse::<f64>().ok())
                .sum();
            let accumulated_nm: f64 = self.mesh_layers[..=i]
                .iter()
                .filter_map(|l| l.thickness_nm.trim().parse::<f64>().ok())
                .sum();
            if accumulated_nm >= total_device_nm {
                return Err(format!(
                    "Accumulated mesh thickness ({accumulated_nm:.3} nm) must be < total device thickness ({total_device_nm:.3} nm)"
                ));
            }
        }
        Ok(())
    }

    fn validate_interface_state(&self, i: usize) -> Result<(), String> {
        let ist = &self.interface_states[i];
        if ist.has_continuous {
            let c = &ist.continuous;
            macro_rules! parse_pos {
                ($v:expr, $label:literal) => {{
                    let v: f64 = $v
                        .trim()
                        .parse()
                        .map_err(|_| format!("{} must be a number", $label))?;
                    if v <= 0.0 {
                        return Err(format!("{} must be > 0", $label));
                    }
                }};
            }
            parse_pos!(c.dit0, "Dit0");
            parse_pos!(c.nssec, "nssec");
            parse_pos!(c.nssev, "nssev");
            parse_pos!(c.ecnl, "|Ec - Ecnl|");
            parse_pos!(c.nd, "nd");
            parse_pos!(c.na, "na");
        }
        if ist.has_discrete {
            let n: usize =
                ist.num_discrete_str.trim().parse().map_err(|_| {
                    "Number of discrete traps must be a positive integer".to_string()
                })?;
            if n == 0 {
                return Err("Number of discrete traps must be ≥ 1".to_string());
            }
        }
        Ok(())
    }

    fn validate_discrete_state(&self, i: usize, j: usize) -> Result<(), String> {
        let trap = &self.interface_states[i].discrete_traps[j];
        macro_rules! parse_pos {
            ($v:expr, $label:literal) => {{
                let v: f64 = $v
                    .trim()
                    .parse()
                    .map_err(|_| format!("{} must be a number", $label))?;
                if v <= 0.0 {
                    return Err(format!("{} must be > 0", $label));
                }
            }};
        }
        parse_pos!(trap.ditmax, "Ditmax");
        parse_pos!(trap.ed, "|Ec - Ed|");
        parse_pos!(trap.fwhm, "FWHM");
        Ok(())
    }

    fn validate_fixed_charges(&self) -> Result<(), String> {
        for (i, v) in self.bulk_charge_densities.iter().enumerate() {
            v.trim().parse::<f64>().map_err(|_| {
                format!(
                    "Bulk charge for layer '{}' must be a number",
                    self.layers[i].name
                )
            })?;
        }
        for (i, v) in self.interface_charge_densities.iter().enumerate() {
            v.trim().parse::<f64>().map_err(|_| {
                format!(
                    "Interface charge {}/{} must be a number",
                    self.layers[i].name,
                    self.layers[i + 1].name
                )
            })?;
        }
        Ok(())
    }

    fn validate_capture_cross_section(&self, k: usize) -> Result<(), String> {
        let ccs = &self.capture_cross_sections[k];
        macro_rules! parse_pos {
            ($v:expr, $label:literal) => {{
                let v: f64 = $v
                    .trim()
                    .parse()
                    .map_err(|_| format!("{} must be a number", $label))?;
                if v <= 0.0 {
                    return Err(format!("{} must be > 0", $label));
                }
            }};
        }
        match ccs.model_type {
            CsModelType::Constant => {
                parse_pos!(ccs.sigma, "Sigma");
            }
            CsModelType::EnergyDependent => {
                parse_pos!(ccs.sigma_mid, "Sigma_mid");
                parse_pos!(ccs.e_mid, "E_mid");
                ccs.e_slope
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| "E_slope must be a number".to_string())?;
            }
        }
        parse_pos!(ccs.mass_electron_coeff, "Effective mass coefficient");
        Ok(())
    }

    fn init_capture_cross_sections(&mut self, active: &[usize]) {
        self.capture_cross_sections = active
            .iter()
            .map(|&iface_id| {
                let mass_coeff = self
                    .layers
                    .get(iface_id + 1)
                    .and_then(|l| l.mass_electron_coeff.trim().parse::<f64>().ok())
                    .filter(|&m| m > 0.0)
                    .unwrap_or(0.2);
                CaptureCrossSectionInput::new_with_default_mass(mass_coeff)
            })
            .collect();
    }

    fn validate_boundary_conditions(&self) -> Result<(), String> {
        self.barrier_height_ev
            .trim()
            .parse::<f64>()
            .map_err(|_| "Barrier height must be a number".to_string())?;
        let bottom_is_sc = self
            .layers
            .last()
            .map(|l| l.is_semiconductor())
            .unwrap_or(false);
        let use_manual = !bottom_is_sc || self.ec_ef_mode == EcEfMode::Manual;
        if use_manual {
            self.ec_ef_bottom_ev
                .trim()
                .parse::<f64>()
                .map_err(|_| "Ec-Ef bottom must be a number".to_string())?;
        } else if compute_equilibrium(self).is_none() {
            return Err(
                "Cannot compute equilibrium potential: check layer mass/donor/temperature values"
                    .into(),
            );
        }
        Ok(())
    }

    // ─── Navigation ───────────────────────────────────────────────────────────

    fn validate_and_advance(&mut self) {
        self.error = None;
        let result: Result<Page, String> = match self.page.clone() {
            Page::SimSettings => self.validate_sim_settings().map(|_| Page::Measurement),
            Page::Measurement => self.validate_measurement().map(|_| Page::StructureCount),
            Page::StructureCount => match self.num_layers_str.trim().parse::<usize>() {
                Ok(n) if n > 0 => {
                    while self.layers.len() < n {
                        let idx = self.layers.len();
                        self.layers.push(LayerInput::new(idx));
                    }
                    self.layers.truncate(n);
                    Ok(Page::Layer(0))
                }
                _ => Err("Number of layers must be ≥ 1".to_string()),
            },
            Page::Layer(i) => self.validate_layer(i).map(|_| {
                if i + 1 < self.layers.len() {
                    Page::Layer(i + 1)
                } else {
                    Page::MeshCount
                }
            }),
            Page::MeshCount => self.validate_mesh_count().map(|_| {
                let n: usize = self.num_mesh_layers_str.trim().parse().unwrap();
                while self.mesh_layers.len() < n {
                    self.mesh_layers.push(MeshLayerInput::new());
                }
                self.mesh_layers.truncate(n);
                Page::MeshLayer(0)
            }),
            Page::MeshLayer(i) => self.validate_mesh_layer(i).map(|_| {
                if i + 1 < self.mesh_layers.len() {
                    Page::MeshLayer(i + 1)
                } else {
                    // Initialize fixed charge arrays when first entering FixedCharge
                    let n = self.layers.len();
                    self.bulk_charge_densities.resize(n, "0".to_string());
                    self.interface_charge_densities
                        .resize(n.saturating_sub(1), "0".to_string());
                    Page::FixedCharge
                }
            }),
            Page::FixedCharge => self.validate_fixed_charges().map(|_| {
                let num_interfaces = self.layers.len().saturating_sub(1);
                self.interface_states
                    .resize(num_interfaces, InterfaceStateInput::new());
                if num_interfaces > 0 {
                    Page::InterfaceStates(0)
                } else {
                    Page::BoundaryConditions
                }
            }),
            Page::InterfaceStates(i) => self.validate_interface_state(i).map(|_| {
                let (has_discrete, n_traps) = {
                    let ist = &self.interface_states[i];
                    (
                        ist.has_discrete,
                        ist.num_discrete_str.trim().parse::<usize>().unwrap_or(0),
                    )
                };
                if has_discrete {
                    self.interface_states[i]
                        .discrete_traps
                        .resize(n_traps, DiscreteStateInput::new());
                    Page::DiscreteState(i, 0)
                } else {
                    let num_interfaces = self.layers.len().saturating_sub(1);
                    if i + 1 < num_interfaces {
                        Page::InterfaceStates(i + 1)
                    } else {
                        let active = active_interface_indices(&self.interface_states);
                        if active.is_empty() {
                            Page::BoundaryConditions
                        } else {
                            self.init_capture_cross_sections(&active);
                            Page::CaptureCrossSection(0)
                        }
                    }
                }
            }),
            Page::DiscreteState(i, j) => self.validate_discrete_state(i, j).map(|_| {
                let num_traps = self.interface_states[i].discrete_traps.len();
                if j + 1 < num_traps {
                    Page::DiscreteState(i, j + 1)
                } else {
                    let num_interfaces = self.layers.len().saturating_sub(1);
                    if i + 1 < num_interfaces {
                        Page::InterfaceStates(i + 1)
                    } else {
                        let active = active_interface_indices(&self.interface_states);
                        if active.is_empty() {
                            Page::BoundaryConditions
                        } else {
                            self.init_capture_cross_sections(&active);
                            Page::CaptureCrossSection(0)
                        }
                    }
                }
            }),
            Page::CaptureCrossSection(k) => self.validate_capture_cross_section(k).map(|_| {
                let active = active_interface_indices(&self.interface_states);
                if k + 1 < active.len() {
                    Page::CaptureCrossSection(k + 1)
                } else {
                    Page::BoundaryConditions
                }
            }),
            Page::BoundaryConditions => self.validate_boundary_conditions().map(|_| Page::Confirm),
            Page::Confirm => return,
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
            Page::MeshCount => {
                let n = self.layers.len();
                if n > 0 {
                    Page::Layer(n - 1)
                } else {
                    Page::StructureCount
                }
            }
            Page::MeshLayer(0) => Page::MeshCount,
            Page::MeshLayer(i) => Page::MeshLayer(i - 1),
            Page::FixedCharge => {
                let n = self.mesh_layers.len();
                if n > 0 {
                    Page::MeshLayer(n - 1)
                } else {
                    Page::MeshCount
                }
            }
            Page::InterfaceStates(0) => Page::FixedCharge,
            Page::InterfaceStates(i) => {
                let prev = i - 1;
                let has_d = self
                    .interface_states
                    .get(prev)
                    .map(|ist| ist.has_discrete)
                    .unwrap_or(false);
                let n_traps = self
                    .interface_states
                    .get(prev)
                    .map(|ist| ist.discrete_traps.len())
                    .unwrap_or(0);
                if has_d && n_traps > 0 {
                    Page::DiscreteState(prev, n_traps - 1)
                } else {
                    Page::InterfaceStates(prev)
                }
            }
            Page::DiscreteState(i, 0) => Page::InterfaceStates(i),
            Page::DiscreteState(i, j) => Page::DiscreteState(i, j - 1),
            Page::CaptureCrossSection(0) => {
                // Go back to the last interface state page
                let num_interfaces = self.layers.len().saturating_sub(1);
                if num_interfaces > 0 {
                    let last_i = num_interfaces - 1;
                    let has_d = self
                        .interface_states
                        .get(last_i)
                        .map(|ist| ist.has_discrete)
                        .unwrap_or(false);
                    let n_traps = self
                        .interface_states
                        .get(last_i)
                        .map(|ist| ist.discrete_traps.len())
                        .unwrap_or(0);
                    if has_d && n_traps > 0 {
                        Page::DiscreteState(last_i, n_traps - 1)
                    } else {
                        Page::InterfaceStates(last_i)
                    }
                } else {
                    Page::FixedCharge
                }
            }
            Page::CaptureCrossSection(k) => Page::CaptureCrossSection(k - 1),
            Page::BoundaryConditions => {
                let active = active_interface_indices(&self.interface_states);
                if !active.is_empty() {
                    Page::CaptureCrossSection(active.len() - 1)
                } else {
                    let num_interfaces = self.layers.len().saturating_sub(1);
                    if num_interfaces > 0 {
                        let last_i = num_interfaces - 1;
                        let has_d = self
                            .interface_states
                            .get(last_i)
                            .map(|ist| ist.has_discrete)
                            .unwrap_or(false);
                        let n_traps = self
                            .interface_states
                            .get(last_i)
                            .map(|ist| ist.discrete_traps.len())
                            .unwrap_or(0);
                        if has_d && n_traps > 0 {
                            Page::DiscreteState(last_i, n_traps - 1)
                        } else {
                            Page::InterfaceStates(last_i)
                        }
                    } else {
                        Page::FixedCharge
                    }
                }
            }
            Page::Confirm => Page::BoundaryConditions,
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
            let dcb = if i == n - 1 {
                0.0
            } else {
                layer.delta_cb_ev.trim().parse().unwrap()
            };
            device_structure.delta_conduction_band.push(dcb);
            if layer.is_semiconductor() {
                device_structure
                    .mass_electron
                    .push(layer.mass_electron_coeff.trim().parse::<f64>().unwrap() * M_ELECTRON);
                device_structure
                    .donor_concentration
                    .push(layer.donor_conc_cm3.trim().parse::<f64>().unwrap() * PER_CM3_TO_PER_M3);
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
            charge_density: self
                .bulk_charge_densities
                .iter()
                .map(|s| s.trim().parse::<f64>().unwrap() * PER_CM3_TO_PER_M3)
                .collect(),
        };
        let interface_fixed_charge = InterfaceFixedCharge {
            interface_id: (0..n.saturating_sub(1) as u32).collect(),
            charge_density: self
                .interface_charge_densities
                .iter()
                .map(|s| s.trim().parse::<f64>().unwrap() * PER_CM2_TO_PER_M2)
                .collect(),
        };
        let mut continuous_interface_states = ContinuousInterfaceStatesConfig {
            interface_id: vec![],
            parameters: vec![],
        };
        let mut discrete_interface_states = DiscreteInterfaceStatesConfig {
            interface_id: vec![],
            parameters: vec![],
        };
        for (i, ist) in self.interface_states.iter().enumerate() {
            if ist.has_continuous {
                let c = &ist.continuous;
                let bandgap =
                    device_structure.bandgap_energy[i].min(device_structure.bandgap_energy[i + 1]);
                continuous_interface_states.interface_id.push(i as u32);
                continuous_interface_states.parameters.push(DIGSModel::new(
                    c.dit0.trim().parse::<f64>().unwrap() * PER_CM2_TO_PER_M2,
                    c.nssec.trim().parse().unwrap(),
                    c.nssev.trim().parse().unwrap(),
                    c.ecnl.trim().parse().unwrap(),
                    c.nd.trim().parse().unwrap(),
                    c.na.trim().parse().unwrap(),
                    bandgap,
                ));
            }
            if ist.has_discrete && !ist.discrete_traps.is_empty() {
                let bandgap =
                    device_structure.bandgap_energy[i].min(device_structure.bandgap_energy[i + 1]);
                discrete_interface_states.interface_id.push(i as u32);
                let traps: Vec<DiscreteModel> = ist
                    .discrete_traps
                    .iter()
                    .map(|t| {
                        DiscreteModel::new(
                            t.ditmax.trim().parse::<f64>().unwrap() * PER_CM2_TO_PER_M2,
                            t.ed.trim().parse().unwrap(),
                            t.fwhm.trim().parse().unwrap(),
                            t.state_type.clone(),
                            bandgap,
                        )
                    })
                    .collect();
                discrete_interface_states.parameters.push(traps);
            }
        }
        let active_ids = active_interface_indices(&self.interface_states);
        let mut ccs_interface_ids = Vec::new();
        let mut ccs_models = Vec::new();
        let mut ccs_masses = Vec::new();
        for (k, &iface_id) in active_ids.iter().enumerate() {
            if let Some(ccs_input) = self.capture_cross_sections.get(k) {
                let model = match ccs_input.model_type {
                    CsModelType::Constant => CaptureCrossSectionModel::Constant {
                        sigma: ccs_input.sigma.trim().parse::<f64>().unwrap() * CM2_TO_M2,
                    },
                    CsModelType::EnergyDependent => CaptureCrossSectionModel::EnergyDependent {
                        sigma_mid: ccs_input.sigma_mid.trim().parse::<f64>().unwrap() * CM2_TO_M2,
                        e_mid: ccs_input.e_mid.trim().parse().unwrap(),
                        e_slope: ccs_input.e_slope.trim().parse().unwrap(),
                    },
                };
                let mass =
                    ccs_input.mass_electron_coeff.trim().parse::<f64>().unwrap() * M_ELECTRON;
                ccs_interface_ids.push(iface_id as u32);
                ccs_models.push(model);
                ccs_masses.push(mass);
            }
        }
        let capture_cross_section = CaptureCrossSectionConfig {
            interface_id: ccs_interface_ids,
            model: ccs_models,
            mass_electron: ccs_masses,
        };

        // Build mesh params: last layer's thickness = total - sum of preceding layers
        let total_thickness_m: f64 = device_structure.thickness.iter().sum();
        let nm = self.mesh_layers.len();
        let mut layer_id = Vec::with_capacity(nm);
        let mut length_per_layer = Vec::with_capacity(nm);
        let mut layer_thickness = Vec::with_capacity(nm);
        let mut accumulated_m = 0.0_f64;
        for (i, ml) in self.mesh_layers.iter().enumerate() {
            layer_id.push(i as u32);
            length_per_layer.push(ml.mesh_length_nm.trim().parse::<f64>().unwrap() * NM_TO_M);
            if i == nm - 1 {
                layer_thickness.push(total_thickness_m - accumulated_m);
            } else {
                let t = ml.thickness_nm.trim().parse::<f64>().unwrap() * NM_TO_M;
                layer_thickness.push(t);
                accumulated_m += t;
            }
        }
        let mesh_params = MeshParams {
            layer_id,
            length_per_layer,
            layer_thickness,
            energy_step: self.energy_step_mev.trim().parse::<f64>().unwrap() * MEV_TO_EV,
        };

        let ec_ef_bottom = {
            let bottom_is_sc = self
                .layers
                .last()
                .map(|l| l.is_semiconductor())
                .unwrap_or(false);
            if bottom_is_sc && self.ec_ef_mode == EcEfMode::Equilibrium {
                compute_equilibrium(&self).expect("equilibrium potential was validated")
            } else {
                self.ec_ef_bottom_ev.trim().parse().unwrap()
            }
        };
        let boundary_conditions = BoundaryConditions {
            barrier_height: self.barrier_height_ev.trim().parse().unwrap(),
            ec_ef_bottom,
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

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Returns the equilibrium potential (Ec-Ef) in eV for the bottom layer when it
/// is a semiconductor and all required values are parseable; `None` otherwise.
fn compute_equilibrium(app: &App) -> Option<f64> {
    let last = app.layers.last()?;
    if !last.is_semiconductor() {
        return None;
    }
    let mass_coeff: f64 = last.mass_electron_coeff.trim().parse().ok()?;
    let nd: f64 = last.donor_conc_cm3.trim().parse().ok()?;
    let temp: f64 = app.temperature.trim().parse().ok()?;
    if mass_coeff <= 0.0 || nd <= 0.0 || temp <= 0.0 {
        return None;
    }
    Some(equilibrium_potential_n_type(
        mass_coeff * M_ELECTRON,
        nd * PER_CM3_TO_PER_M3,
        temp,
    ))
}

/// Compute the total device thickness in nm from parsed layer inputs.
fn total_device_nm(app: &App) -> Option<f64> {
    let mut sum = 0.0_f64;
    for l in &app.layers {
        sum += l.thickness_nm.trim().parse::<f64>().ok()?;
    }
    Some(sum)
}

/// Compute the accumulated mesh thickness in nm for layers 0..=i.
fn accumulated_mesh_nm(app: &App, up_to: usize) -> f64 {
    app.mesh_layers[..up_to]
        .iter()
        .filter_map(|ml| ml.thickness_nm.trim().parse::<f64>().ok())
        .sum()
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
        fields.push(("Donor Concentration (cm⁻³)", layer.donor_conc_cm3.clone(), false));
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
            "Donor Concentration (cm^-3)",
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
        let acc_nm = accumulated_mesh_nm(app, i);
        let remaining = total_nm - acc_nm;
        format!(
            "\n  Auto thickness (remaining): {remaining:.3} nm\n  (= total {total_nm:.3} nm - accumulated {acc_nm:.3} nm)"
        )
    } else {
        let total_nm = total_device_nm(app).unwrap_or(0.0);
        let acc_nm = accumulated_mesh_nm(app, i);
        let after = accumulated_mesh_nm(app, i + 1);
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
