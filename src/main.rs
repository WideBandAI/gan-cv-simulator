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

use crate::config::config_source::select_config_source;
use crate::mesh::mesh_builder as mb;
use crate::solvers::cv_solver::CVSolver;
use crate::solvers::poisson_solver::PoissonSolver;
use crate::utils::save_configuration;
use colored::*;

fn print_layer_structure(device: &config::structure::DeviceStructure) {
    use crate::constants::units::{M_TO_NM, PER_M3_TO_PER_CM3};

    let col_widths: [usize; 6] = [15, 13, 10, 8, 8, 14];
    let headers = ["Layer", "Material", "Thick(nm)", "Eg(eV)", "dEc(eV)", "Nd(cm^-3)"];

    let sep: String = {
        let inner = col_widths
            .iter()
            .map(|&w| "-".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("+");
        format!("+{}+", inner)
    };

    println!("\n{}", sep);
    let header_row: String = headers
        .iter()
        .zip(col_widths.iter())
        .map(|(h, &w)| format!(" {:^w$} ", h, w = w))
        .collect::<Vec<_>>()
        .join("|");
    println!("|{}|", header_row);
    println!("{}", sep);

    for i in 0..device.id.len() {
        let mat = match device.material_type[i] {
            config::structure::MaterialType::Semiconductor => "Semiconductor",
            config::structure::MaterialType::Insulator => "Insulator",
        };
        println!(
            "| {:<15} | {:<13} | {:>10.2} | {:>8.3} | {:>8.3} | {:>14.3e} |",
            device.name[i],
            mat,
            device.thickness[i] * M_TO_NM,
            device.bandgap_energy[i],
            device.delta_conduction_band[i],
            device.donor_concentration[i] * PER_M3_TO_PER_CM3,
        );
    }

    println!("{}\n", sep);
}

fn print_interface_states(
    device: &config::structure::DeviceStructure,
    continuous: &config::interface_states::ContinuousInterfaceStatesConfig,
    discrete: &config::interface_states::DiscreteInterfaceStatesConfig,
) {
    use crate::constants::units::PER_M2_TO_PER_CM2;

    if continuous.interface_id.is_empty() && discrete.interface_id.is_empty() {
        return;
    }

    if !continuous.interface_id.is_empty() {
        let col_widths: [usize; 7] = [23, 14, 6, 6, 11, 6, 6];
        let headers = ["Interface", "Dit0(cm^-2)", "nssec", "nssev", "Ec-Ecnl(eV)", "nd", "na"];
        let sep: String = {
            let inner = col_widths
                .iter()
                .map(|&w| "-".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("+");
            format!("+{}+", inner)
        };
        println!("\n{}", sep);
        let header_row: String = headers
            .iter()
            .zip(col_widths.iter())
            .map(|(h, &w)| format!(" {:^w$} ", h, w = w))
            .collect::<Vec<_>>()
            .join("|");
        println!("|{}|", header_row);
        println!("{}", sep);
        for (idx, &iface_id) in continuous.interface_id.iter().enumerate() {
            let i = iface_id as usize;
            let iface_name = format!("{} / {}", device.name[i], device.name[i + 1]);
            let p = &continuous.parameters[idx];
            println!(
                "| {:<23} | {:>14.3e} | {:>6.2} | {:>6.2} | {:>11.3} | {:>6.2} | {:>6.2} |",
                iface_name,
                p.dit0 * PER_M2_TO_PER_CM2,
                p.nssec,
                p.nssev,
                p.ecnl,
                p.nd,
                p.na,
            );
        }
        println!("{}\n", sep);
    }

    if !discrete.interface_id.is_empty() {
        let col_widths: [usize; 6] = [23, 5, 14, 10, 8, 13];
        let headers = [
            "Interface",
            "Trap#",
            "Ditmax(cm^-2)",
            "|Ec-Ed|(eV)",
            "FWHM(eV)",
            "Type",
        ];
        let sep: String = {
            let inner = col_widths
                .iter()
                .map(|&w| "-".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("+");
            format!("+{}+", inner)
        };
        println!("\n{}", sep);
        let header_row: String = headers
            .iter()
            .zip(col_widths.iter())
            .map(|(h, &w)| format!(" {:^w$} ", h, w = w))
            .collect::<Vec<_>>()
            .join("|");
        println!("|{}|", header_row);
        println!("{}", sep);
        for (idx, &iface_id) in discrete.interface_id.iter().enumerate() {
            let i = iface_id as usize;
            let iface_name = format!("{} / {}", device.name[i], device.name[i + 1]);
            for (trap_idx, model) in discrete.parameters[idx].iter().enumerate() {
                println!(
                    "| {:<23} | {:>5} | {:>14.3e} | {:>10.3} | {:>8.3} | {:<13} |",
                    iface_name,
                    trap_idx,
                    model.ditmax() * PER_M2_TO_PER_CM2,
                    model.ed(),
                    model.fwhm(),
                    model.state_type().to_string(),
                );
            }
        }
        println!("{}\n", sep);
    }
}

fn main() -> anyhow::Result<()> {
    println!("\n{}\n", "GaN C-V Simulator".green().bold());
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

    print_layer_structure(&config.device_structure);
    print_interface_states(
        &config.device_structure,
        &config.continuous_interface_states,
        &config.discrete_interface_states,
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
