use super::types::{
    active_interface_indices, CaptureCrossSectionInput, CsModelType,
    DiscreteStateInput, EcEfMode, InterfaceStateInput, LayerInput, MeshLayerInput, Page,
};
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

// ─── App state ────────────────────────────────────────────────────────────────

pub(crate) struct App {
    pub(crate) page: Page,
    pub(crate) focused: usize,
    pub(crate) error: Option<String>,

    // SimSettings
    pub(crate) sim_name: String,
    pub(crate) sor_factor: String,
    pub(crate) convergence: String,
    pub(crate) max_iter: String,
    pub(crate) parallel: bool,

    // Measurement
    pub(crate) temperature: String,
    pub(crate) v_start: String,
    pub(crate) v_end: String,
    pub(crate) v_step: String,
    pub(crate) ac_voltage: String,
    pub(crate) meas_time: String,
    pub(crate) stress_voltage: String,
    pub(crate) stress_relief_voltage: String,
    pub(crate) stress_relief_time: String,

    // Device structure
    pub(crate) num_layers_str: String,
    pub(crate) layers: Vec<LayerInput>,

    // Mesh
    pub(crate) num_mesh_layers_str: String,
    pub(crate) energy_step_mev: String,
    pub(crate) mesh_layers: Vec<MeshLayerInput>,

    // Fixed charges
    pub(crate) bulk_charge_densities: Vec<String>,      // C/cm^3 per layer
    pub(crate) interface_charge_densities: Vec<String>, // C/cm^2 per interface

    // Interface states
    pub(crate) interface_states: Vec<InterfaceStateInput>,

    // Capture cross section (one per active interface, in same order as active_interface_indices)
    pub(crate) capture_cross_sections: Vec<CaptureCrossSectionInput>,

    // Boundary conditions
    pub(crate) barrier_height_ev: String,
    pub(crate) ec_ef_mode: EcEfMode,
    pub(crate) ec_ef_bottom_ev: String,
}

impl App {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn next_field(&mut self) {
        let n = self.field_count();
        if n > 0 {
            self.focused = (self.focused + 1) % n;
            self.error = None;
        }
    }

    pub(crate) fn prev_field(&mut self) {
        let n = self.field_count();
        if n > 0 {
            self.focused = (self.focused + n - 1) % n;
            self.error = None;
        }
    }

    // ─── Toggle ───────────────────────────────────────────────────────────────

    pub(crate) fn is_toggle(&self) -> bool {
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

    pub(crate) fn toggle_focused(&mut self) {
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

    pub(crate) fn type_char(&mut self, c: char) {
        if let Some(f) = self.active_text_field() {
            f.push(c);
        }
    }

    pub(crate) fn backspace(&mut self) {
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

    pub(crate) fn validate_and_advance(&mut self) {
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

    pub(crate) fn go_back(&mut self) {
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

    pub(crate) fn build_config(self) -> ConfigurationBuilder {
        let sim_settings = SimSettings {
            sim_name: self.sim_name.trim().to_string(),
            sor_relaxation_factor: self
                .sor_factor
                .trim()
                .parse()
                .expect("Validation should have ensured this is a valid number."),
            convergence_criterion: self
                .convergence
                .trim()
                .parse()
                .expect("Validation should have ensured this is a valid number."),
            max_iterations: self
                .max_iter
                .trim()
                .parse()
                .expect("Validation should have ensured this is a valid number."),
            parallel_use: self.parallel,
        };

        let measurement = Measurement {
            temperature: Temperature {
                temperature: self
                    .temperature
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number."),
            },
            voltage: Voltage {
                start: self
                    .v_start
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number."),
                end: self
                    .v_end
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number."),
                step: self
                    .v_step
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number."),
            },
            ac_voltage: self
                .ac_voltage
                .trim()
                .parse::<f64>()
                .expect("Validation should have ensured this is a valid number.")
                * MV_TO_V,
            time: Time {
                measurement_time: self
                    .meas_time
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number."),
            },
            stress: Stress {
                stress_voltage: self
                    .stress_voltage
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number."),
                stress_relief_voltage: self
                    .stress_relief_voltage
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number."),
                stress_relief_time: self
                    .stress_relief_time
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number."),
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
            device_structure.thickness.push(
                layer
                    .thickness_nm
                    .trim()
                    .parse::<f64>()
                    .expect("Validation should have ensured this is a valid number.")
                    * NM_TO_M,
            );
            device_structure.permittivity.push(
                layer
                    .permittivity
                    .trim()
                    .parse::<f64>()
                    .expect("Validation should have ensured this is a valid number.")
                    * EPSILON_0,
            );
            device_structure.bandgap_energy.push(
                layer
                    .bandgap_ev
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number."),
            );
            let dcb = if i == n - 1 {
                0.0
            } else {
                layer
                    .delta_cb_ev
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number.")
            };
            device_structure.delta_conduction_band.push(dcb);
            if layer.is_semiconductor() {
                device_structure.mass_electron.push(
                    layer
                        .mass_electron_coeff
                        .trim()
                        .parse::<f64>()
                        .expect("Validation should have ensured this is a valid number.")
                        * M_ELECTRON,
                );
                device_structure.donor_concentration.push(
                    layer
                        .donor_conc_cm3
                        .trim()
                        .parse::<f64>()
                        .expect("Validation should have ensured this is a valid number.")
                        * PER_CM3_TO_PER_M3,
                );
                device_structure.energy_level_donor.push(
                    layer
                        .energy_donor_ev
                        .trim()
                        .parse()
                        .expect("Validation should have ensured this is a valid number."),
                );
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
                .map(|s| {
                    s.trim()
                        .parse::<f64>()
                        .expect("Validation should have ensured this is a valid number.")
                        * PER_CM3_TO_PER_M3
                })
                .collect(),
        };
        let interface_fixed_charge = InterfaceFixedCharge {
            interface_id: (0..n.saturating_sub(1) as u32).collect(),
            charge_density: self
                .interface_charge_densities
                .iter()
                .map(|s| {
                    s.trim()
                        .parse::<f64>()
                        .expect("Validation should have ensured this is a valid number.")
                        * PER_CM2_TO_PER_M2
                })
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
                    c.dit0
                        .trim()
                        .parse::<f64>()
                        .expect("Validation should have ensured this is a valid number.")
                        * PER_CM2_TO_PER_M2,
                    c.nssec
                        .trim()
                        .parse()
                        .expect("Validation should have ensured this is a valid number."),
                    c.nssev
                        .trim()
                        .parse()
                        .expect("Validation should have ensured this is a valid number."),
                    c.ecnl
                        .trim()
                        .parse()
                        .expect("Validation should have ensured this is a valid number."),
                    c.nd.trim()
                        .parse()
                        .expect("Validation should have ensured this is a valid number."),
                    c.na.trim()
                        .parse()
                        .expect("Validation should have ensured this is a valid number."),
                    bandgap,
                ));
            }
            if ist.has_discrete && !ist.discrete_traps.is_empty() {
                let bandgap =
                    device_structure.bandgap_energy[i].min(device_structure.bandgap_energy[i + 1]);
                discrete_interface_states.interface_id.push(i as u32);
                let traps: Vec<DiscreteModel> =
                    ist.discrete_traps
                        .iter()
                        .map(|t| {
                            DiscreteModel::new(
                                t.ditmax.trim().parse::<f64>().expect(
                                    "Validation should have ensured this is a valid number.",
                                ) * PER_CM2_TO_PER_M2,
                                t.ed.trim().parse().expect(
                                    "Validation should have ensured this is a valid number.",
                                ),
                                t.fwhm.trim().parse().expect(
                                    "Validation should have ensured this is a valid number.",
                                ),
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
                let model =
                    match ccs_input.model_type {
                        CsModelType::Constant => {
                            CaptureCrossSectionModel::Constant {
                                sigma: ccs_input.sigma.trim().parse::<f64>().expect(
                                    "Validation should have ensured this is a valid number.",
                                ) * CM2_TO_M2,
                            }
                        }
                        CsModelType::EnergyDependent => {
                            CaptureCrossSectionModel::EnergyDependent {
                                sigma_mid: ccs_input.sigma_mid.trim().parse::<f64>().expect(
                                    "Validation should have ensured this is a valid number.",
                                ) * CM2_TO_M2,
                                e_mid: ccs_input.e_mid.trim().parse().expect(
                                    "Validation should have ensured this is a valid number.",
                                ),
                                e_slope: ccs_input.e_slope.trim().parse().expect(
                                    "Validation should have ensured this is a valid number.",
                                ),
                            }
                        }
                    };
                let mass = ccs_input
                    .mass_electron_coeff
                    .trim()
                    .parse::<f64>()
                    .expect("Validation should have ensured this is a valid number.")
                    * M_ELECTRON;
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
            length_per_layer.push(
                ml.mesh_length_nm
                    .trim()
                    .parse::<f64>()
                    .expect("Validation should have ensured this is a valid number.")
                    * NM_TO_M,
            );
            if i == nm - 1 {
                layer_thickness.push(total_thickness_m - accumulated_m);
            } else {
                let t = ml
                    .thickness_nm
                    .trim()
                    .parse::<f64>()
                    .expect("Validation should have ensured this is a valid number.")
                    * NM_TO_M;
                layer_thickness.push(t);
                accumulated_m += t;
            }
        }
        let mesh_params = MeshParams {
            layer_id,
            length_per_layer,
            layer_thickness,
            energy_step: self
                .energy_step_mev
                .trim()
                .parse::<f64>()
                .expect("Validation should have ensured this is a valid number.")
                * MEV_TO_EV,
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
                self.ec_ef_bottom_ev
                    .trim()
                    .parse()
                    .expect("Validation should have ensured this is a valid number.")
            }
        };
        let boundary_conditions = BoundaryConditions {
            barrier_height: self
                .barrier_height_ev
                .trim()
                .parse()
                .expect("Validation should have ensured this is a valid number."),
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
pub(crate) fn compute_equilibrium(app: &App) -> Option<f64> {
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
pub(crate) fn total_device_nm(app: &App) -> Option<f64> {
    let mut sum = 0.0_f64;
    for l in &app.layers {
        sum += l.thickness_nm.trim().parse::<f64>().ok()?;
    }
    Some(sum)
}

/// Compute the accumulated mesh thickness in nm for layers 0..up_to.
pub(crate) fn accumulated_mesh_nm(app: &App, up_to: usize) -> f64 {
    app.mesh_layers[..up_to]
        .iter()
        .filter_map(|ml| ml.thickness_nm.trim().parse::<f64>().ok())
        .sum()
}
