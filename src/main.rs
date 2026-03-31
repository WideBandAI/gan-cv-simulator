pub mod config;
pub mod constants;
pub mod mesh_builder;
pub mod physics_equations;
pub mod plot;
pub mod save_files;
pub mod solvers;
pub mod utils;

use crate::constants::simulation::INITIAL_POTENTIAL;

use std::fs;

use crate::config::configuration_builder::ConfigurationBuilder;
use crate::mesh_builder::mesh_builder as mb;
use crate::solvers::cv_solver::CVSolver;
use crate::solvers::poisson_solver::PoissonSolver;

fn main() -> anyhow::Result<()> {
    println!("GaN C-V Simulator");
    let config = ConfigurationBuilder::from_interactive().build();
    println!("{:#?}", config);
    let output_dir = format!("outputs/{}", config.sim_settings.sim_name);
    fs::create_dir_all(&output_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create output directory '{}': {}. Please check permissions and try again.",
            output_dir,
            e
        )
    })?;

    let mesh_structure = mb::build(&config);
    let poisson_solver = PoissonSolver::new(
        mesh_structure,
        INITIAL_POTENTIAL,
        config.measurement.temperature.temperature,
        config.sim_settings.sor_relaxation_factor,
        config.sim_settings.convergence_criterion,
        config.sim_settings.max_iterations,
        config.sim_settings.parallel_use,
        config.capture_cross_section.thermal_velocity,
    );
    let mut cv_solver = CVSolver::new(
        poisson_solver,
        config.measurement,
        config.boundary_conditions,
        output_dir,
    );
    cv_solver.run()?;
    Ok(())
}
