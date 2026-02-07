pub mod cli;
pub mod constants;
pub mod physics_equations;
pub mod utils;

use cli::parameters_definition;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let parameters = parameters_definition::ParametersDefinition::define();
    println!("{:#?}", parameters);
    println!("Simulation complete.");
}
