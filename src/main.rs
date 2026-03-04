pub mod config;
pub mod constants;
pub mod mesh_builder;
pub mod physics_equations;
pub mod solvers;
pub mod utils;

use crate::constants::simulation::INITIAL_POTENTIAL;

use crate::config::configuration_builder::ConfigurationBuilder;
use crate::mesh_builder::mesh_builder as mb;
use crate::solvers::cv_solver::CVSolver;
use crate::solvers::poisson_solver::PoissonSolver;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let config = ConfigurationBuilder::from_interactive().build();
    println!("{:#?}", config);
    let mesh_structure = mb::build(&config);
    let solver = PoissonSolver::new(
        mesh_structure,
        INITIAL_POTENTIAL,
        config.measurement.temperature.temperature,
        config.sim_settings.damping_factor,
        config.sim_settings.convergence_criterion,
        config.sim_settings.max_iterations,
    );
    let mut cv_solver = CVSolver::new(solver, config.measurement, config.boundary_conditions);
    cv_solver.run();
}
