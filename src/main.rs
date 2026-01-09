pub mod structure;

use clap::Parser;
use structure::DeviceParameters;

/// 1D GaN C-V Simulator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(flatten)]
    simulation_params: SimulationParameters,

    #[clap(flatten)]
    device_params: DeviceParameters,
}

#[derive(Parser, Debug)]
struct SimulationParameters {
    /// Start voltage (V)
    #[arg(short, long)]
    start_voltage: f64,

    /// End voltage (V)
    #[arg(short, long)]
    end_voltage: f64,

    /// Voltage step (V)
    #[arg(long, default_value_t = 0.1)]
    step: f64,

    /// Temperature (K)
    #[arg(long, default_value_t = 300.0)]
    temperature: f64,
}

fn main() {
    let cli = Cli::parse();
    println!("Starting C-V simulation with the following parameters:");
    println!("{:#?}", cli);
}
