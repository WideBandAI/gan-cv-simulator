pub mod cli;
pub mod constants;
pub mod electrostatics;
pub mod mesh_builder;
pub mod physics_equations;
pub mod utils;

use crate::cli::configuration_builder::ConfigurationBuilder;
use crate::electrostatics::poisson_solver::PoissonSolver;
use crate::mesh_builder::mesh_builder as mb;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let config = ConfigurationBuilder::from_interactive().build();
    println!("{:#?}", config);
    let mesh_structure = mb::build(&config);
    println!("{:#?}", mesh_structure);
    let mut solver = PoissonSolver::new(
        mesh_structure,
        1.0,
        config.measurement.temperature.temperature,
        1.9,
        1e-6,
        10000,
    );
    solver.set_boundary_conditions(
        config.measurement.voltage.start,
        config.boundary_conditions.barrier_height,
        config.boundary_conditions.ec_ef_bottom,
    );
    println!("{:#?}", solver);
    solver.solve_poisson();
    println!("Simulation complete.");
}
