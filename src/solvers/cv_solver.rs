use crate::config::boundary_conditions::BoundaryConditions;
use crate::config::measurement::Measurement;
use crate::constants::physics::Q_ELECTRON;
use crate::constants::units::{F_TO_NF, M2_TO_CM2};
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
        let start = self.measurement.voltage.start;
        let end = self.measurement.voltage.end;
        let step = self.measurement.voltage.step;

        if step == 0.0 {
            panic!("voltage step cannot be zero");
        }

        let mut gate_voltage = start;
        let forward = step > 0.0;

        while (forward && gate_voltage <= end) || (!forward && gate_voltage >= end) {
            let capacitance = self.solve_cv(gate_voltage);
            println!(
                "Gate Voltage: {:<10.3} V, Capacitance: {:.3e} nF/cm^2\n",
                gate_voltage,
                capacitance * F_TO_NF * M2_TO_CM2
            );
            gate_voltage += step;
        }
    }

    fn solve_cv(&mut self, gate_voltage: f64) -> f64 {
        _ = self.total_charge_at_vg(gate_voltage); // Update the potential profile for the given gate voltage
        let charge_plus = self.total_charge_at_vg(gate_voltage + self.measurement.ac_voltage);
        let charge_minus = self.total_charge_at_vg(gate_voltage - self.measurement.ac_voltage);

        // Capacitance C = |dQ/dV|
        let capacitance = (charge_plus - charge_minus).abs() / (2.0 * self.measurement.ac_voltage);

        capacitance
    }

    fn total_charge_at_vg(&mut self, gate_voltage: f64) -> f64 {
        self.set_gate_voltage(gate_voltage);
        self.poisson_solver.solve_poisson();

        let potential_profile = self.poisson_solver.get_potential_profile();
        let mut total_charge = 0.0; // in C/m^2
        let n_nodes = potential_profile.depth.len();

        for idx in 0..n_nodes {
            let h_up = if idx > 0 {
                potential_profile.depth[idx] - potential_profile.depth[idx - 1]
            } else {
                0.0
            };
            let h_low = if idx < n_nodes - 1 {
                potential_profile.depth[idx + 1] - potential_profile.depth[idx]
            } else {
                0.0
            };
            let delta_x = (h_up + h_low) / 2.0;

            let n_s = potential_profile.electron_density[idx];

            let rho = Q_ELECTRON * (-n_s);
            total_charge += rho * delta_x;
        }

        total_charge
    }

    fn set_gate_voltage(&mut self, gate_voltage: f64) {
        self.poisson_solver.set_boundary_conditions(
            -gate_voltage + self.boundary_conditions.barrier_height,
            self.boundary_conditions.ec_ef_bottom,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::measurement::{Stress, Temperature, Time, Voltage};
    use crate::constants::physics::EPSILON_0;
    use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure, IDX};
    use approx::relative_eq;

    fn make_cv_mesh(
        mass_electron: f64,
        permittivity: f64,
        donor_concentration: f64,
        bulk_fixcharge: f64,
    ) -> MeshStructure {
        let n = 6;
        MeshStructure {
            id: vec![
                IDX::Surface,
                IDX::Bulk(0),
                IDX::Bulk(0),
                IDX::Bulk(0),
                IDX::Bulk(0),
                IDX::Bottom,
            ],
            depth: vec![0.0, 1e-9, 2e-9, 3e-9, 4e-9, 5e-9],
            mass_electron: vec![
                0.0,
                mass_electron,
                mass_electron,
                mass_electron,
                mass_electron,
                0.0,
            ],
            permittivity: vec![
                0.0,
                permittivity,
                permittivity,
                permittivity,
                permittivity,
                0.0,
            ],
            delta_conduction_band: vec![0.0; n],
            donor_concentration: vec![
                0.0,
                donor_concentration,
                donor_concentration,
                donor_concentration,
                donor_concentration,
                0.0,
            ],
            energy_level_donor: vec![0.0, 0.05, 0.05, 0.05, 0.05, 0.0],
            fixcharge_density: vec![
                FixChargeDensity::Bulk(0.0),
                FixChargeDensity::Bulk(bulk_fixcharge),
                FixChargeDensity::Bulk(bulk_fixcharge),
                FixChargeDensity::Bulk(bulk_fixcharge),
                FixChargeDensity::Bulk(bulk_fixcharge),
                FixChargeDensity::Bulk(0.0),
            ],
        }
    }

    fn make_measurement(start: f64, end: f64, step: f64, ac_voltage: f64) -> Measurement {
        Measurement {
            temperature: Temperature { temperature: 300.0 },
            voltage: Voltage { start, end, step },
            ac_voltage,
            time: Time {
                measurement_time: 100.0,
            },
            stress: Stress {
                stress_voltage: 0.0,
                stress_relief_voltage: 0.0,
                stress_relief_time: 0.0,
            },
        }
    }

    fn make_boundary_conditions(barrier_height: f64, ec_ef_bottom: f64) -> BoundaryConditions {
        BoundaryConditions {
            barrier_height,
            ec_ef_bottom,
        }
    }

    fn make_cv_solver(
        mass_electron: f64,
        donor_concentration: f64,
        barrier_height: f64,
        ec_ef_bottom: f64,
        voltage_start: f64,
        voltage_end: f64,
        voltage_step: f64,
        ac_voltage: f64,
    ) -> CVSolver {
        let eps = 10.0 * EPSILON_0;
        let mesh = make_cv_mesh(mass_electron, eps, donor_concentration, 0.0);
        let poisson_solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-8, 100_000);
        let measurement = make_measurement(voltage_start, voltage_end, voltage_step, ac_voltage);
        let bc = make_boundary_conditions(barrier_height, ec_ef_bottom);
        CVSolver::new(poisson_solver, measurement, bc)
    }

    #[test]
    fn test_new_initializes_fields_correctly() {
        let eps = 10.0 * EPSILON_0;
        let mesh = make_cv_mesh(0.2, eps, 1e22, 0.0);
        let poisson_solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000);
        let measurement = make_measurement(-2.0, 2.0, 0.1, 0.02);
        let bc = make_boundary_conditions(1.0, 0.1);

        let cv_solver = CVSolver::new(poisson_solver, measurement, bc);

        assert!(relative_eq!(
            cv_solver.boundary_conditions.barrier_height,
            1.0
        ));
    }

    #[test]
    fn test_run_forward_sweep_completes() {
        let mut cv_solver = make_cv_solver(0.0, 0.0, 1.0, 0.5, 0.0, 0.2, 0.1, 0.02);
        cv_solver.run();
    }
}
