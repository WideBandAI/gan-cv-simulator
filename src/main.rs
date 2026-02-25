pub mod config;
pub mod constants;
pub mod mesh_builder;
pub mod physics_equations;
pub mod solvers;
pub mod utils;

use crate::constants::simulation::INITIAL_POTENTIAL;

use crate::config::configuration_builder::ConfigurationBuilder;
use crate::mesh_builder::mesh_builder as mb;
use crate::solvers::poisson_solver::PoissonSolver;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let config = ConfigurationBuilder::from_interactive().build();
    println!("{:#?}", config);
    let mesh_structure = mb::build(&config);
    let mut solver = PoissonSolver::new(
        mesh_structure,
        INITIAL_POTENTIAL,
        config.measurement.temperature.temperature,
        config.sim_settings.sor_relaxation_factor,
        config.sim_settings.convergence_criterion,
        config.sim_settings.max_iterations,
    );
    solver.set_boundary_conditions(
        config.measurement.voltage.start,
        config.boundary_conditions.barrier_height,
        config.boundary_conditions.ec_ef_bottom,
    );
    solver.solve_poisson();
    println!("Simulation complete.");
    let potential_profile = solver.get_potential_profile();
    // output profile to CSV
    let output_file = "potential_profile.csv";
    if let Err(e) = utils::write_potential_profile_csv(output_file, &potential_profile) {
        eprintln!("Failed to write potential profile CSV: {}", e);
    } else {
        println!("Potential profile saved to {}", output_file);
    }
}
