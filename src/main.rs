pub mod config;
pub mod constants;
pub mod mesh;
pub mod physics_equations;
pub mod plot;
pub mod save_files;
pub mod solvers;
pub mod utils;

use crate::constants::simulation::INITIAL_POTENTIAL;

use std::fs;

use crate::config::configuration_builder::ConfigurationBuilder;
use crate::mesh::mesh_builder as mb;
use crate::solvers::cv_solver::CVSolver;
use crate::solvers::poisson_solver::PoissonSolver;
use crate::utils::save_configuration;

fn list_config_files(config_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(config_dir) else {
        return vec![];
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    files.sort();
    files
}

fn select_config_source() -> anyhow::Result<ConfigurationBuilder> {
    println!("Select configuration source:");
    println!("  [1] Interactive input");
    println!("  [2] Load from config file");

    loop {
        let mut input = String::new();
        print!("Enter choice (default: 1): ");
        std::io::Write::flush(&mut std::io::stdout())?;
        std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut input)?;
        match input.trim() {
            "" | "1" => return Ok(ConfigurationBuilder::from_interactive()),
            "2" => {
                let config_dir = std::path::Path::new("config");
                let files = list_config_files(config_dir);
                if files.is_empty() {
                    println!("No config files found in '{}'. Falling back to interactive input.", config_dir.display());
                    return Ok(ConfigurationBuilder::from_interactive());
                }
                println!("Available config files:");
                for (i, path) in files.iter().enumerate() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    println!("  [{}] {}", i + 1, name);
                }
                loop {
                    let mut sel = String::new();
                    print!("Enter number (1-{}): ", files.len());
                    std::io::Write::flush(&mut std::io::stdout())?;
                    std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut sel)?;
                    let sel = sel.trim();
                    if let Ok(n) = sel.parse::<usize>()
                        && n >= 1 && n <= files.len() {
                            let path = &files[n - 1];
                            println!("Loading config from '{}'...", path.display());
                            return ConfigurationBuilder::from_json(path);
                        }
                    println!("Invalid selection. Please enter a number between 1 and {}.", files.len());
                }
            }
            _ => println!("Invalid choice. Please enter 1 or 2."),
        }
    }
}

fn main() -> anyhow::Result<()> {
    println!("GaN C-V Simulator");
    let config = select_config_source()?.build();
    println!("{:#?}", config);
    let output_dir = format!("outputs/{}", config.sim_settings.sim_name);
    fs::create_dir_all(&output_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create output directory '{}': {}. Please check permissions and try again.",
            output_dir,
            e
        )
    })?;

    let config_dir = std::path::Path::new("config");
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
