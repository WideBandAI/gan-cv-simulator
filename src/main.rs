pub mod cli;
pub mod constants;
pub mod utils;

use cli::device_definition;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let structure = device_definition::DeviceDefinition::define();
    println!("{:#?}", structure);
    println!("Simulation complete.");
}
