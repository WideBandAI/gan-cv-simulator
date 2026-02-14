pub mod cli;
pub mod constants;
pub mod mesh_builder;
pub mod physics_equations;
pub mod utils;

use crate::cli::configuration_builder::ConfigurationBuilder;
use crate::mesh_builder::mesh_builder::MeshBuilder;
fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let config = ConfigurationBuilder::from_interactive().build();
    println!("{:#?}", config);
    let mesh_structure = MeshBuilder::build(&config);
    println!("{:#?}", mesh_structure);
    println!("Simulation complete.");
}
