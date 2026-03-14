pub enum TrapStatesType {
    DonorLike(f64),
    AcceptorLike(f64),
}

pub struct DIGSModel {
    dit0: f64,
    nssec: f64,
    nssev: f64,
    ecnl: f64,
    nd: f64,
    na: f64,
    bandgap: f64,
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
    /// use crate::...;
    ///
    /// let _ = contunious_states();
    /// ```
    pub fn contunious_states(&self, potential: f64) -> TrapStatesType {
        if potential > self.bandgap {
            panic!("potential cannot be greater than bandgap")
        } else if potential > self.ecnl {
            // donorlike interface states
            let e0d = (self.bandgap - self.ecnl) * self.nssev.ln().powf(-1.0 / self.nd);
            let dit = self.dit0 * ((-potential + self.ecnl).abs() / e0d).powf(self.nd).exp();
            TrapStatesType::DonorLike(dit)
        } else {
            // acceptorlike interface states
            let e0a = self.ecnl * self.nssec.ln().powf(-1.0 / self.na);
            // Discrete interface states を含める場合はここに処理を追加
            let dit = self.dit0 * ((-potential + self.ecnl).abs() / e0a).powf(self.na).exp();
            TrapStatesType::AcceptorLike(dit)
        }
    }
}

pub enum DiscreteStateType {
    DonorLike,
    AcceptorLike,
}

pub struct DiscreteModel {
    ditmax: f64,
    ed: f64,
    fwhm: f64,
    state_type: DiscreteStateType,
}

impl DiscreteModel {
    pub fn new(ditmax: f64, ed: f64, fwhm: f64, state_type: DiscreteStateType) -> Self {
        Self {
            ditmax,
            ed,
            fwhm,
            state_type,
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
    /// - `f64` - Describe the return value.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = discrete_states();
    /// ```
    pub fn discrete_states(&self, potential: f64) -> TrapStatesType {
        let sigma = self.fwhm.powi(2) / (4.0 * 2.0_f64.ln());
        let dit = self.ditmax * (-(potential - self.ed).powi(2) / sigma).exp();
        if self.state_type is DiscreteStateType::DonorLike {
            TrapStatesType::DonorLike((dit))
        } else {
            TrapStatesType::AcceptorLike((dit))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contunious_states_donorlike() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential > ecnl, donorlike
        let potential = 2.0;
        match model.contunious_states(potential) {
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
    fn test_contunious_states_acceptorlike() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential <= ecnl, acceptorlike
        let potential = 1.0;
        match model.contunious_states(potential) {
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
    #[should_panic(expected = "potential cannot be greater than bandgap")]
    fn test_contunious_states_potential_greater_than_bandgap() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential > bandgap, should panic
        model.contunious_states(3.1);
    }

    #[test]
    fn test_contunious_states_at_ecnl() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential == ecnl, acceptorlike
        let potential = model.ecnl;
        match model.contunious_states(potential) {
            TrapStatesType::AcceptorLike(_) => {}
            _ => panic!("Expected AcceptorLike at ecnl"),
        }
    }

    #[test]
    fn test_contunious_states_at_bandgap() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential == bandgap, donorlike
        let potential = model.bandgap;
        match model.contunious_states(potential) {
            TrapStatesType::DonorLike(_) => {}
            _ => panic!("Expected DonorLike at bandgap"),
        }
    }

    #[test]
    fn test_minimamu_dit_is_dit0() {
        let model = DIGSModel::new(1.0, 2.0, 3.0, 1.5, 2.0, 2.5, 3.0);
        // potential == ecnl, acceptorlike
        let potential = model.ecnl;
        match model.contunious_states(potential) {
            TrapStatesType::AcceptorLike(dit) => {
                assert_eq!(dit, model.dit0);
            }
            _ => panic!("Expected AcceptorLike at ecnl"),
        }
    }
}
