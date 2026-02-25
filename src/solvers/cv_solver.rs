use crate::config::boundary_conditions::BoundaryConditions;
use crate::config::measurement::Measurement;
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

    pub fn run(&mut self) {
        for gate_voltage in (self.measurement.voltage.start as i64
            ..=self.measurement.voltage.end as i64)
            .step_by(self.measurement.voltage.step as usize)
        {
            let capacitance = self.solve_cv(gate_voltage as f64);
            println!(
                "Gate Voltage: {} V, Capacitance: {} F",
                gate_voltage, capacitance
            );
        }
    }

    fn solve_cv(&mut self, gate_voltage: f64) -> f64 {
        println!(
            "Solving C-V characteristics for gate voltage {} V and AC voltage {} V",
            gate_voltage, self.measurement.ac_voltage
        );
        self.electron_density_at_vg(gate_voltage);
        let electron_density_vg_plus_ac =
            self.electron_density_at_vg(gate_voltage + self.measurement.ac_voltage);
        let electron_density_vg_minus_ac =
            self.electron_density_at_vg(gate_voltage - self.measurement.ac_voltage);

        let capacitance = (electron_density_vg_plus_ac - electron_density_vg_minus_ac)
            / (2.0 * self.measurement.ac_voltage);

        capacitance
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.poisson_solver.temperature = temperature;
    }

    fn electron_density_at_vg(&mut self, gate_voltage: f64) -> f64 {
        self.set_gate_voltage(gate_voltage);
        self.poisson_solver.solve_poisson();
        let potential_at_vg = self.poisson_solver.get_potential_profile();
        potential_at_vg
            .iter()
            .map(|(_, _, electron_density, _)| *electron_density)
            .sum()
    }

    fn set_gate_voltage(&mut self, gate_voltage: f64) {
        self.poisson_solver.set_boundary_conditions(
            -gate_voltage + self.boundary_conditions.barrier_height,
            self.boundary_conditions.ec_ef_bottom,
        );
    }
}
