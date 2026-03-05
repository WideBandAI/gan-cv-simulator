use crate::utils::get_input;
use crate::utils::get_parsed_input_with_default;

#[derive(Debug)]
pub struct SimSettings {
    pub sim_name: String,
    pub sor_relaxation_factor: f64,
    pub convergence_criterion: f64,
    pub max_iterations: usize,
    pub parallel_use: bool,
}

fn get_bool_input(prompt: &str) -> bool {
    loop {
        let input = get_input(prompt);
        match input.trim().to_lowercase().as_str() {
            "y" => return true,
            "n" => return false,
            _ => println!("Invalid input. Please enter 'y' or 'n'."),
        }
    }
}

pub fn define_sim_settings() -> SimSettings {
    println!("Define simulation settings.");
    let sim_name: String = get_input("Enter a name for this simulation: ");

    let sor_relaxation_factor: f64 =
        get_parsed_input_with_default("Enter the SOR relaxation factor. Default is 1.9: ", 1.9);
    let convergence_criterion: f64 = get_parsed_input_with_default(
        "Enter the convergence criterion (in eV). Default is 1e-6: ",
        1e-6,
    );
    let max_iterations: usize = get_parsed_input_with_default(
        "Enter the maximum number of iterations. Default is 10000: ",
        500000,
    );
    let parallel_use: bool =
        get_bool_input("Use parallel processing for Poisson solver? (y/n). Default is n: ");

    SimSettings {
        sim_name,
        sor_relaxation_factor,
        convergence_criterion,
        max_iterations,
        parallel_use,
    }
}
