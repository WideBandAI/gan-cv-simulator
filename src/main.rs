pub mod cli;
pub mod constants;
pub mod physics_equations;
pub mod utils;

use cli::configuration_builder;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let config = configuration_builder::ConfigurationBuilder::run();
    println!("{:#?}", config);
    println!("Simulation complete.");
}
