use crate::config::structure::MaterialType;
use crate::physics_equations::interface_states::DiscreteStateType;

// ─── Enums ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EcEfMode {
    Manual,
    Equilibrium,
}

// ─── Interface states input structs ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct ContinuousStateInput {
    pub(crate) dit0: String,
    pub(crate) nssec: String,
    pub(crate) nssev: String,
    pub(crate) ecnl: String,
    pub(crate) nd: String,
    pub(crate) na: String,
}

impl ContinuousStateInput {
    pub(crate) fn new() -> Self {
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
pub(crate) struct DiscreteStateInput {
    pub(crate) ditmax: String,
    pub(crate) ed: String,
    pub(crate) fwhm: String,
    pub(crate) state_type: DiscreteStateType,
}

impl DiscreteStateInput {
    pub(crate) fn new() -> Self {
        Self {
            ditmax: "1e12".to_string(),
            ed: "0.5".to_string(),
            fwhm: "0.3".to_string(),
            state_type: DiscreteStateType::DonorLike,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InterfaceStateInput {
    pub(crate) has_continuous: bool,
    pub(crate) continuous: ContinuousStateInput,
    pub(crate) has_discrete: bool,
    pub(crate) num_discrete_str: String,
    pub(crate) discrete_traps: Vec<DiscreteStateInput>,
}

impl InterfaceStateInput {
    pub(crate) fn new() -> Self {
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
pub(crate) enum CsModelType {
    Constant,
    EnergyDependent,
}

#[derive(Debug, Clone)]
pub(crate) struct CaptureCrossSectionInput {
    pub(crate) model_type: CsModelType,
    pub(crate) sigma: String,
    pub(crate) sigma_mid: String,
    pub(crate) e_mid: String,
    pub(crate) e_slope: String,
    pub(crate) mass_electron_coeff: String,
}

impl CaptureCrossSectionInput {
    pub(crate) fn new_with_default_mass(mass_coeff: f64) -> Self {
        Self {
            model_type: CsModelType::Constant,
            sigma: "1e-16".to_string(),
            sigma_mid: "1e-16".to_string(),
            e_mid: "0.5".to_string(),
            e_slope: "0.1".to_string(),
            mass_electron_coeff: format!("{:.4}", mass_coeff),
        }
    }

    pub(crate) fn field_count(&self) -> usize {
        match self.model_type {
            CsModelType::Constant => 3,
            CsModelType::EnergyDependent => 5,
        }
    }
}

// ─── Pages ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Page {
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
pub(crate) fn active_interface_indices(interface_states: &[InterfaceStateInput]) -> Vec<usize> {
    interface_states
        .iter()
        .enumerate()
        .filter(|(_, ist)| ist.has_continuous || ist.has_discrete)
        .map(|(i, _)| i)
        .collect()
}

// ─── Input state structs ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct LayerInput {
    pub(crate) name: String,
    pub(crate) material: MaterialType,
    pub(crate) thickness_nm: String,
    pub(crate) permittivity: String,
    pub(crate) bandgap_ev: String,
    pub(crate) delta_cb_ev: String,
    pub(crate) mass_electron_coeff: String,
    pub(crate) donor_conc_cm3: String,
    pub(crate) energy_donor_ev: String,
}

impl LayerInput {
    pub(crate) fn new(index: usize) -> Self {
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

    pub(crate) fn is_semiconductor(&self) -> bool {
        self.material == MaterialType::Semiconductor
    }

    pub(crate) fn field_count(&self, is_last: bool) -> usize {
        let base = if self.is_semiconductor() { 9 } else { 6 };
        if is_last { base - 1 } else { base }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MeshLayerInput {
    pub(crate) mesh_length_nm: String,
    pub(crate) thickness_nm: String,
}

impl MeshLayerInput {
    pub(crate) fn new() -> Self {
        Self {
            mesh_length_nm: "0.1".to_string(),
            thickness_nm: String::new(),
        }
    }

    /// Non-last layer: 2 fields (mesh_length, thickness).
    /// Last layer: 1 field (mesh_length only; thickness is auto-calculated).
    pub(crate) fn field_count(is_last: bool) -> usize {
        if is_last { 1 } else { 2 }
    }
}
