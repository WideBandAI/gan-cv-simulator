pub mod config;
pub mod constants;
pub mod mesh;
pub mod physics_equations;
pub mod plot;
pub mod save_files;
pub mod solvers;
pub mod tui;
pub mod utils;

use crate::constants::simulation::{CONFIG_DIR, INITIAL_POTENTIAL};

use std::fs;

use crate::mesh::mesh_builder as mb;
use crate::solvers::cv_solver::CVSolver;
use crate::solvers::poisson_solver::PoissonSolver;
use crate::utils::save_configuration;

fn main() -> anyhow::Result<()> {
    let config = tui::run()?.build();
    println!("{:#?}", config);

    let output_dir = format!("outputs/{}", config.sim_settings.sim_name);
    fs::create_dir_all(&output_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create output directory '{}': {}. Please check permissions and try again.",
            output_dir,
            e
        )
    })?;

    let config_dir = std::path::Path::new(CONFIG_DIR);
    std::fs::create_dir_all(config_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create config directory '{}': {}. Please check permissions and try again.",
            config_dir.display(),
            e
        )
    })?;

    let sanitized_sim_name = config
        .sim_settings
        .sim_name
        .replace(['/', '\\'], "_")
        .replace("..", "_");
    let config_filename = format!("{}.json", sanitized_sim_name);
    let global_config_path = config_dir.join(&config_filename);
    save_configuration(&config, &global_config_path)?;

    let output_config_path = std::path::Path::new(&output_dir).join(&config_filename);
    save_configuration(&config, &output_config_path)?;

    println!(
        "Configuration saved to '{}' and '{}'.",
        global_config_path.display(),
        output_config_path.display()
    );

    let mesh_structure = mb::build(&config);
    let poisson_solver = PoissonSolver::new(
        mesh_structure,
        INITIAL_POTENTIAL,
        config.measurement.temperature.temperature,
        config.sim_settings.sor_relaxation_factor,
        config.sim_settings.convergence_criterion,
        config.sim_settings.max_iterations,
        config.sim_settings.parallel_use,
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
