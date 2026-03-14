pub enum TrapStatesType {
    DonorLike(f64),
    AcceptorLike(f64),
}

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

    /// エネルギー b を引数に取り、Dit を計算して f64 で返す機能
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
