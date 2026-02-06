pub mod constants;
pub mod structure;
pub mod utils;

use structure::device_definition;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let structure = device_definition::DeviceDefinition::define();
    println!("{:#?}", structure);
    println!("Simulation complete.");
}
