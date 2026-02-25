use crate::config::boundary_conditions::BoundaryConditions;
use crate::config::measurement::Measurement;
use crate::config::structure::DeviceStructure;
use crate::solvers::poisson_solver::PoissonSolver;

#[derive(Debug)]
pub struct CVSolver {
    pub poisson_solver: PoissonSolver,
    pub measurement: Measurement,
    pub boundary_conditions: BoundaryConditions,
    pub device_structure: DeviceStructure,
}

impl CVSolver {
    pub fn new(
        poisson_solver: PoissonSolver,
        measurement: Measurement,
        boundary_conditions: BoundaryConditions,
        device_structure: DeviceStructure,
    ) -> Self {
        Self {
            poisson_solver,
            measurement,
            boundary_conditions,
            device_structure,
        }
    }

    pub fn solve_cv(&mut self, gate_voltage: f64, ac_voltage: f64) {
        // Placeholder for C-V simulation logic
        // This would involve iterating over the voltage range defined in the measurement,
        // solving Poisson's equation at each voltage step, and calculating the capacitance.
        println!(
            "Solving C-V characteristics for gate voltage {} V and AC voltage {} V",
            gate_voltage, ac_voltage
        );
        self.poisson_solver
            .set_boundary_conditions(gate_voltage, 0.0); // Example boundary conditions
        self.poisson_solver.solve_poisson();
        let potential_at_vg = self.poisson_solver.get_potential_profile();

        self.poisson_solver
            .set_boundary_conditions(gate_voltage + ac_voltage, 0.0); // Example boundary conditions for AC voltage
        self.poisson_solver.solve_poisson();
        let potential_at_vg_plus_ac = self.poisson_solver.get_potential_profile();

        self.poisson_solver
            .set_boundary_conditions(gate_voltage - ac_voltage, 0.0); // Example boundary conditions for AC voltage
        self.poisson_solver.solve_poisson();
        let potential_at_vg_minus_ac = self.poisson_solver.get_potential_profile();
    }

    pub fn set_gate_voltage(&mut self, gate_voltage: f64) {
        self.poisson_solver
            .set_boundary_conditions(gate_voltage, 0.0);
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.poisson_solver.temperature = temperature;
    }

    fn solve_poisson(&mut self) {
        self.poisson_solver.solve_poisson();
    }
}
