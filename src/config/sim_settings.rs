use crate::utils::get_parsed_input_with_default;

#[derive(Debug)]
pub struct SimSettings {
    pub damping_factor: f64,
    pub convergence_criterion: f64,
    pub max_iterations: usize,
}

pub fn define_sim_settings() -> SimSettings {
    println!("Define simulation settings.");

    let damping_factor: f64 =
        get_parsed_input_with_default("Enter the Newton damping factor. Default is 1.0: ", 1.0);
    let convergence_criterion: f64 = get_parsed_input_with_default(
        "Enter the convergence criterion (in eV). Default is 1e-6: ",
        1e-6,
    );
    let max_iterations: usize = get_parsed_input_with_default(
        "Enter the maximum number of iterations. Default is 100: ",
        100,
    );

    SimSettings {
        damping_factor,
        convergence_criterion,
        max_iterations,
    }
}
