pub mod cli;
pub mod constants;
pub mod mesh_builder;
pub mod physics_equations;
pub mod utils;

use cli::configuration_builder;
use mesh_builder::mesh_builder::MeshBuilder;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let config = configuration_builder::ConfigurationBuilder::from_interactive().build();
    println!("{:#?}", config);
    let mesh_builder = MeshBuilder::run(&config);
    // println!("{:#?}", mesh_builder);
    println!("Simulation complete.");
}
