use crate::utils::get_parsed_input_with_default;

#[derive(Debug)]
pub struct SimSettings {
    pub sor_relaxation_factor: f64,
    pub convergence_criterion: f64,
    pub max_iterations: usize,
}

pub fn define_sim_settings() -> SimSettings {
    println!("Define simulation settings.");

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

    SimSettings {
        sor_relaxation_factor,
        convergence_criterion,
        max_iterations,
    }
}
