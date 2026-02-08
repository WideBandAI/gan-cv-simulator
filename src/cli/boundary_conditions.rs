use crate::utils::{get_parsed_input, get_parsed_input_with_default};

#[derive(Debug)]
pub struct BoundaryConditions {
    pub barrier_height: f64,
    pub back_gate_voltage: f64,
}

pub fn define_boundary_conditions() -> BoundaryConditions {
    println!("Define boundary conditions.");

    let barrier_height: f64 =
        get_parsed_input_with_default("Enter the barrier height (in eV). Default is 1.0: ", 1.0);
    let back_gate_voltage: f64 =
        get_parsed_input_with_default("Enter the back gate voltage (in V). Default is 0.0: ", 0.0);

    BoundaryConditions {
        barrier_height,
        back_gate_voltage,
    }
}
