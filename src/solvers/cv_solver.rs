use crate::config::boundary_conditions::BoundaryConditions;
use crate::config::measurement::Measurement;
use crate::constants::physics::Q_ELECTRON;
use crate::solvers::poisson_solver::PoissonSolver;

#[derive(Debug)]
pub struct CVSolver {
    pub poisson_solver: PoissonSolver,
    pub measurement: Measurement,
    pub boundary_conditions: BoundaryConditions,
}

impl CVSolver {
    pub fn new(
        poisson_solver: PoissonSolver,
        measurement: Measurement,
        boundary_conditions: BoundaryConditions,
    ) -> Self {
        Self {
            poisson_solver,
            measurement,
            boundary_conditions,
        }
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.poisson_solver.temperature = temperature;
    }

    pub fn run(&mut self) {
        // perform basic validation of the step size before iterating
        let start = self.measurement.voltage.start;
        let end = self.measurement.voltage.end;
        let step = self.measurement.voltage.step;

        if step == 0.0 {
            panic!("voltage step cannot be zero");
        }

        // determine loop direction based on sign of step
        let mut gate_voltage = start;
        let forward = step > 0.0;

        while (forward && gate_voltage <= end) || (!forward && gate_voltage >= end) {
            let capacitance = self.solve_cv(gate_voltage);
            println!(
                "Gate Voltage: {:<10.3} V, Capacitance: {:.3e} F\n",
                gate_voltage, capacitance
            );
            gate_voltage += step;
        }
    }

    fn solve_cv(&mut self, gate_voltage: f64) -> f64 {
        self.electron_density_at_vg(gate_voltage);
        let electron_density_vg_plus_ac =
            self.electron_density_at_vg(gate_voltage + self.measurement.ac_voltage);
        let electron_density_vg_minus_ac =
            self.electron_density_at_vg(gate_voltage - self.measurement.ac_voltage);

        let capacitance = Q_ELECTRON * (electron_density_vg_plus_ac - electron_density_vg_minus_ac)
            / (2.0 * self.measurement.ac_voltage);

        capacitance
    }

    fn electron_density_at_vg(&mut self, gate_voltage: f64) -> f64 {
        self.set_gate_voltage(gate_voltage);
        self.poisson_solver.solve_poisson();
        let potential_at_vg = self.poisson_solver.get_potential_profile();
        let mut total_electron_density = 0.0; // in m2
        for idx in 0..potential_at_vg.depth.len() {
            if potential_at_vg.electron_density[idx] > 0.0 {
                let upper_mesh_length = potential_at_vg.depth[idx] - potential_at_vg.depth[idx - 1];
                let lower_mesh_length = potential_at_vg.depth[idx + 1] - potential_at_vg.depth[idx];
                let mesh_length = (upper_mesh_length + lower_mesh_length) / 2.0;
                total_electron_density += potential_at_vg.electron_density[idx] * mesh_length;
            }
        }
        total_electron_density
    }

    fn set_gate_voltage(&mut self, gate_voltage: f64) {
        self.poisson_solver.set_boundary_conditions(
            -gate_voltage + self.boundary_conditions.barrier_height,
            self.boundary_conditions.ec_ef_bottom,
        );
    }
}
