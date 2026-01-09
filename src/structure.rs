use serde::{Deserialize, Serialize};
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Serialize, Deserialize)]
pub struct DeviceParameters {
    #[clap(flatten)]
    pub insulator: Insulator,
    #[clap(flatten)]
    pub semiconductor: Semiconductor,
    #[clap(flatten)]
    pub gate: Gate,
}

#[derive(Parser, Debug, Serialize, Deserialize)]
pub struct Insulator {
    /// Insulator thickness (nm)
    #[arg(long, default_value_t = 10.0)]
    pub thickness: f64,

    /// Insulator relative permittivity
    #[arg(long, default_value_t = 9.0)]
    pub relative_permittivity: f64,
}

#[derive(Parser, Debug, Serialize, Deserialize)]
pub struct Semiconductor {
    /// Semiconductor layer thickness (um)
    #[arg(long, default_value_t = 1.0)]
    pub thickness: f64,

    /// Doping type
    #[arg(long, value_enum, default_value_t = DopingType::N)]
    pub doping_type: DopingType,
    
    /// Doping concentration (cm^-3)
    #[arg(long, default_value_t = 1e17)]
    pub doping_concentration: f64,

    /// Band gap (eV)
    #[arg(long, default_value_t = 3.4)]
    pub band_gap: f64,

    /// Semiconductor relative permittivity
    #[arg(long, default_value_t = 9.7)]
    pub relative_permittivity: f64,

    /// Electron affinity (eV)
    #[arg(long, default_value_t = 4.1)]
    pub electron_affinity: f64,
}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum DopingType {
    N,
    P,
}

#[derive(Parser, Debug, Serialize, Deserialize)]
pub struct Gate {
    /// Gate metal work function (eV)
    #[arg(long, default_value_t = 5.1)]
    pub work_function: f64,
}
