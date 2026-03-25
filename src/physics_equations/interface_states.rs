use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum TrapStatesType {
    DonorLike(f64),
    AcceptorLike(f64),
}

#[derive(Debug, PartialEq)]
pub enum PotentialError {
    GreaterThanBandgap,
    Negative,
}

#[derive(Debug, Clone, Copy)]
pub struct DIGSModel {
    pub dit0: f64,
    pub nssec: f64,
    pub nssev: f64,
    pub ecnl: f64,
    pub nd: f64,
    pub na: f64,
    pub bandgap: f64,
}

impl DIGSModel {
    pub fn new(
        dit0: f64,
        nssec: f64,
        nssev: f64,
        ecnl: f64,
        nd: f64,
        na: f64,
        bandgap: f64,
    ) -> Self {
        Self {
            dit0,
            nssec,
            nssev,
            ecnl,
            nd,
            na,
            bandgap,
        }
    }

    /// Contunious interface states.
    ///
    /// # Arguments
    ///
    /// - `potential` (`f64`) - |E - Ec| in eV.
    ///
    /// # Returns
    ///
    /// - `TrapStatesType` - Trap States Type.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::physics_equations::interface_states::DIGSModel;
    /// use crate::physics_equations::interface_states::TrapStatesType;
    ///
    /// let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
    /// let potential = 1.0;
    /// let trap_states = model.continuous_states(potential).unwrap();
    /// ```
    pub fn continuous_states(&self, potential: f64) -> Result<TrapStatesType, PotentialError> {
        if potential > self.bandgap {
            Err(PotentialError::GreaterThanBandgap)
        } else if potential < 0.0 {
            Err(PotentialError::Negative)
        } else if potential > self.ecnl {
            // donorlike interface states
            let e0d = (self.bandgap - self.ecnl) * self.nssev.ln().powf(-1.0 / self.nd);
            let dit = self.dit0 * ((-potential + self.ecnl).abs() / e0d).powf(self.nd).exp();
            Ok(TrapStatesType::DonorLike(dit))
        } else {
            // acceptorlike interface states
            let e0a = self.ecnl * self.nssec.ln().powf(-1.0 / self.na);
            let dit = self.dit0 * ((-potential + self.ecnl).abs() / e0a).powf(self.na).exp();
            Ok(TrapStatesType::AcceptorLike(dit))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DiscreteStateType {
    DonorLike,
    AcceptorLike,
}

impl FromStr for DiscreteStateType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DonorLike" => Ok(DiscreteStateType::DonorLike),
            "AcceptorLike" => Ok(DiscreteStateType::AcceptorLike),
            _ => Err(anyhow::anyhow!("Invalid DiscreteStateType: {}", s)),
        }
    }
}

impl fmt::Display for DiscreteStateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DonorLike => write!(f, "DonorLike"),
            Self::AcceptorLike => write!(f, "AcceptorLike"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscreteModel {
    ditmax: f64,
    ed: f64,
    fwhm: f64,
    state_type: DiscreteStateType,
    pub bandgap: f64,
}

impl DiscreteModel {
    pub fn new(
        ditmax: f64,
        ed: f64,
        fwhm: f64,
        state_type: DiscreteStateType,
        bandgap: f64,
    ) -> Self {
        Self {
            ditmax,
            ed,
            fwhm,
            state_type,
            bandgap,
        }
    }

    /// Discrete interface states.
    ///
    /// # Arguments
    ///
    /// - `potential` (`f64`) - |E - Ec| in eV.
    ///
    /// # Returns
    ///
    /// - `TrapStatesType` - The calculated trap state type and density.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::physics_equations::interface_states::DiscreteModel;
    /// use crate::physics_equations::interface_states::DiscreteStateType;
    /// use crate::physics_equations::interface_states::TrapStatesType;
    ///
    /// let model = DiscreteModel::new(1.0, 2.0, 3.0, DiscreteStateType::DonorLike);
    /// let potential = 1.0;
    /// let trap_states = model.discrete_states(potential);
    /// ```
    pub fn discrete_states(&self, potential: f64) -> Result<TrapStatesType, PotentialError> {
        if potential > self.bandgap {
            Err(PotentialError::GreaterThanBandgap)
        } else if potential < 0.0 {
            Err(PotentialError::Negative)
        } else {
            let sigma = self.fwhm.powi(2) / (4.0 * 2.0_f64.ln());
            let dit = self.ditmax * (-(potential - self.ed).powi(2) / sigma).exp();
            if self.state_type == DiscreteStateType::DonorLike {
                Ok(TrapStatesType::DonorLike(dit))
            } else {
                Ok(TrapStatesType::AcceptorLike(dit))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuous_states_donorlike() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential > ecnl, donorlike
        let potential = 2.0;
        match model.continuous_states(potential).unwrap() {
            TrapStatesType::DonorLike(dit) => {
                let e0d = (model.bandgap - model.ecnl) * model.nssev.ln().powf(-1.0 / model.nd);
                let expected_dit =
                    model.dit0 * ((-potential + model.ecnl).abs() / e0d).powf(model.nd).exp();
                assert!((dit - expected_dit).abs() < 1e-10);
            }
            _ => panic!("Expected DonorLike"),
        }
    }

    #[test]
    fn test_continuous_states_acceptorlike() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential <= ecnl, acceptorlike
        let potential = 1.0;
        match model.continuous_states(potential).unwrap() {
            TrapStatesType::AcceptorLike(dit) => {
                let e0a = model.ecnl * model.nssec.ln().powf(-1.0 / model.na);
                let expected_dit =
                    model.dit0 * ((-potential + model.ecnl).abs() / e0a).powf(model.na).exp();
                assert!((dit - expected_dit).abs() < 1e-10);
            }
            _ => panic!("Expected AcceptorLike"),
        }
    }

    #[test]
    fn test_continuous_states_potential_greater_than_bandgap() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential > bandgap, should return error
        let result = model.continuous_states(3.1);
        assert_eq!(result.unwrap_err(), PotentialError::GreaterThanBandgap);
    }

    #[test]
    fn test_continuous_states_at_ecnl() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential == ecnl, acceptorlike
        let potential = model.ecnl;
        match model.continuous_states(potential).unwrap() {
            TrapStatesType::AcceptorLike(_) => {}
            _ => panic!("Expected AcceptorLike at ecnl"),
        }
    }

    #[test]
    fn test_continuous_states_at_bandgap() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential == bandgap, donorlike
        let potential = model.bandgap;
        match model.continuous_states(potential).unwrap() {
            TrapStatesType::DonorLike(_) => {}
            _ => panic!("Expected DonorLike at bandgap"),
        }
    }

    #[test]
    fn test_minimum_dit_is_dit0() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential == ecnl, acceptorlike
        let potential = model.ecnl;
        match model.continuous_states(potential).unwrap() {
            TrapStatesType::AcceptorLike(dit) => {
                assert_eq!(dit, model.dit0);
            }
            _ => panic!("Expected AcceptorLike at ecnl"),
        }
    }

    #[test]
    fn test_discrete_states_donorlike() {
        let ditmax = 1.0;
        let ed = 1.5;
        let fwhm = 0.2;
        let model = DiscreteModel::new(ditmax, ed, fwhm, DiscreteStateType::DonorLike);
        let potential = 1.6;
        match model.discrete_states(potential) {
            TrapStatesType::DonorLike(dit) => {
                let sigma = fwhm.powi(2) / (4.0 * 2.0_f64.ln());
                let expected_dit = ditmax * (-(potential - ed).powi(2) / sigma).exp();
                assert!((dit - expected_dit).abs() < 1e-10);
            }
            _ => panic!("Expected DonorLike"),
        }
    }

    #[test]
    fn test_discrete_states_acceptorlike() {
        let ditmax = 2.0;
        let ed = 1.0;
        let fwhm = 0.3;
        let model = DiscreteModel::new(ditmax, ed, fwhm, DiscreteStateType::AcceptorLike);
        let potential = 0.8;
        match model.discrete_states(potential) {
            TrapStatesType::AcceptorLike(dit) => {
                let sigma = fwhm.powi(2) / (4.0 * 2.0_f64.ln());
                let expected_dit = ditmax * (-(potential - ed).powi(2) / sigma).exp();
                assert!((dit - expected_dit).abs() < 1e-10);
            }
            _ => panic!("Expected AcceptorLike"),
        }
    }

    #[test]
    fn test_discrete_states_peak_value() {
        let ditmax = 3.0;
        let ed = 2.0;
        let fwhm = 0.1;
        let model = DiscreteModel::new(ditmax, ed, fwhm, DiscreteStateType::DonorLike);
        let potential = ed; // peak
        match model.discrete_states(potential) {
            TrapStatesType::DonorLike(dit) => {
                assert!((dit - ditmax).abs() < 1e-10);
            }
            _ => panic!("Expected DonorLike"),
        }
    }
}
