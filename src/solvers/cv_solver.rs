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

/// C-V solver
///
/// # Arguments
///
/// - `poisson_solver` (`PoissonSolver`) - Poisson solver
/// - `measurement` (`Measurement`) - Measurement parameters
/// - `boundary_conditions` (`BoundaryConditions`) - Boundary conditions for the solver
///
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let _ = new();
/// ```
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

    /// Run the C-V calculation
    ///
    /// # Arguments
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = run();
    /// ```
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
                "Gate Voltage: {:<10.3} V, Capacitance: {:.3e} nF/cm^2\n",
                gate_voltage,
                capacitance * F_TO_NF * M2_TO_CM2
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

    /// Get electron density (/m^2) at gate voltage
    ///
    /// This fuction calculates the electron density (/m^3) to electron density (/m^2) at the gate voltage.
    /// electron density (/m^3) * mesh length (m) = electron density (/m^2)
    ///
    /// # Arguments
    ///
    /// - `gate_voltage` (`f64`) - Gate voltage in volts.
    ///
    /// # Returns
    ///
    /// - `f64` - Electron density in m^-2.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = electron_density_at_vg();
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::measurement::{Stress, Temperature, Time, Voltage};
    use crate::constants::physics::EPSILON_0;
    use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure, IDX};
    use approx::relative_eq;

    // -----------------------------------------------------------------------
    // Helper: テスト用の MeshStructure を手動で作成
    //
    // ノード構成:
    //   [0] Surface    depth=0.0
    //   [1] Bulk(0)    depth=1e-9
    //   [2] Bulk(0)    depth=2e-9
    //   [3] Bulk(0)    depth=3e-9
    //   [4] Bulk(0)    depth=4e-9
    //   [5] Bottom     depth=5e-9
    // -----------------------------------------------------------------------
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

    // -----------------------------------------------------------------------
    // new()
    // -----------------------------------------------------------------------

    /// new() を呼んだとき、各フィールドが正しく設定されること
    #[test]
    fn test_new_initializes_fields_correctly() {
        let eps = 10.0 * EPSILON_0;
        let mesh = make_cv_mesh(0.2, eps, 1e22, 0.0);
        let poisson_solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000);
        let measurement = make_measurement(-2.0, 2.0, 0.1, 0.02);
        let bc = make_boundary_conditions(1.0, 0.1);

        let cv_solver = CVSolver::new(poisson_solver, measurement, bc);

        assert!(
            relative_eq!(
                cv_solver.boundary_conditions.barrier_height,
                1.0,
                epsilon = 1e-15
            ),
            "barrier_height mismatch"
        );
        assert!(
            relative_eq!(
                cv_solver.boundary_conditions.ec_ef_bottom,
                0.1,
                epsilon = 1e-15
            ),
            "ec_ef_bottom mismatch"
        );
        assert!(
            relative_eq!(cv_solver.measurement.voltage.start, -2.0, epsilon = 1e-15),
            "voltage start mismatch"
        );
        assert!(
            relative_eq!(cv_solver.measurement.voltage.end, 2.0, epsilon = 1e-15),
            "voltage end mismatch"
        );
        assert!(
            relative_eq!(cv_solver.measurement.voltage.step, 0.1, epsilon = 1e-15),
            "voltage step mismatch"
        );
        assert!(
            relative_eq!(cv_solver.measurement.ac_voltage, 0.02, epsilon = 1e-15),
            "ac_voltage mismatch"
        );
    }

    // -----------------------------------------------------------------------
    // set_temperature()
    // -----------------------------------------------------------------------

    /// set_temperature() で PoissonSolver の温度が更新されること
    #[test]
    fn test_set_temperature_updates_poisson_solver() {
        let mut cv_solver = make_cv_solver(0.2, 1e22, 1.0, 0.1, 0.0, 1.0, 0.1, 0.02);
        assert!(
            relative_eq!(cv_solver.poisson_solver.temperature, 300.0, epsilon = 1e-15),
            "initial temperature should be 300.0"
        );

        cv_solver.set_temperature(400.0);
        assert!(
            relative_eq!(cv_solver.poisson_solver.temperature, 400.0, epsilon = 1e-15),
            "temperature should be updated to 400.0"
        );
    }

    // -----------------------------------------------------------------------
    // set_gate_voltage()
    // -----------------------------------------------------------------------

    /// set_gate_voltage() が PoissonSolver に正しい境界条件を設定すること
    /// surface_potential = -gate_voltage + barrier_height - delta_Ec[0]
    /// bottom_potential  = ec_ef_bottom - delta_Ec[last]
    #[test]
    fn test_set_gate_voltage_sets_boundary_conditions() {
        let barrier_height = 1.2;
        let ec_ef_bottom = 0.3;
        let mut cv_solver =
            make_cv_solver(0.0, 0.0, barrier_height, ec_ef_bottom, 0.0, 1.0, 0.1, 0.02);

        let gate_voltage = 0.5;
        cv_solver.set_gate_voltage(gate_voltage);

        // surface_potential = -gate_voltage + barrier_height - delta_Ec[0]
        // delta_Ec[0] = 0.0 なので surface_potential = -0.5 + 1.2 = 0.7
        let expected_surface = -gate_voltage + barrier_height;
        assert!(
            relative_eq!(
                cv_solver.poisson_solver.potential.potential[0],
                expected_surface,
                epsilon = 1e-15
            ),
            "surface potential: {} != {}",
            cv_solver.poisson_solver.potential.potential[0],
            expected_surface
        );

        // bottom_potential = ec_ef_bottom - delta_Ec[last]
        // delta_Ec[last] = 0.0 なので bottom_potential = 0.3
        let n = cv_solver.poisson_solver.mesh_structure.id.len();
        assert!(
            relative_eq!(
                cv_solver.poisson_solver.potential.potential[n - 1],
                ec_ef_bottom,
                epsilon = 1e-15
            ),
            "bottom potential: {} != {}",
            cv_solver.poisson_solver.potential.potential[n - 1],
            ec_ef_bottom
        );
    }

    /// 負のゲート電圧で surface potential が大きくなること
    #[test]
    fn test_set_gate_voltage_negative_vg() {
        let barrier_height = 1.0;
        let ec_ef_bottom = 0.1;
        let mut cv_solver =
            make_cv_solver(0.0, 0.0, barrier_height, ec_ef_bottom, 0.0, 1.0, 0.1, 0.02);

        let gate_voltage = -2.0;
        cv_solver.set_gate_voltage(gate_voltage);

        // surface = -(-2.0) + 1.0 = 3.0
        let expected_surface = 3.0;
        assert!(
            relative_eq!(
                cv_solver.poisson_solver.potential.potential[0],
                expected_surface,
                epsilon = 1e-15
            ),
            "surface potential: {} != {}",
            cv_solver.poisson_solver.potential.potential[0],
            expected_surface
        );
    }

    // -----------------------------------------------------------------------
    // electron_density_at_vg()
    // -----------------------------------------------------------------------

    /// mass_electron=0 → 有効状態密度 Nc=0 → 電子密度がゼロになること
    #[test]
    fn test_electron_density_at_vg_zero_mass_gives_zero() {
        let mut cv_solver = make_cv_solver(
            0.0, // mass_electron = 0 → electron_density = 0
            0.0, // donor_concentration = 0
            1.0, // barrier_height
            0.1, // ec_ef_bottom
            0.0, 1.0, 0.1, 0.02,
        );

        let electron_density = cv_solver.electron_density_at_vg(0.5);
        assert!(
            relative_eq!(electron_density, 0.0, epsilon = 1e-30),
            "electron density should be zero with mass_electron=0: {}",
            electron_density
        );
    }

    /// 高い正のゲート電圧ではポテンシャルが下がり、電子密度が増加すること
    #[test]
    fn test_electron_density_increases_with_positive_gate_voltage() {
        let mut cv_solver = make_cv_solver(
            0.2,  // mass_electron
            1e22, // donor_concentration
            1.0,  // barrier_height
            0.1,  // ec_ef_bottom
            0.0, 5.0, 0.1, 0.02,
        );

        // 低いゲート電圧での電子密度
        let n_low = cv_solver.electron_density_at_vg(0.0);
        // 高いゲート電圧での電子密度
        let n_high = cv_solver.electron_density_at_vg(5.0);

        assert!(
            n_high >= n_low,
            "electron density should increase or stay same with higher positive gate voltage: n_low={}, n_high={}",
            n_low,
            n_high
        );
    }

    // -----------------------------------------------------------------------
    // solve_cv()
    // -----------------------------------------------------------------------

    /// mass_electron=0 → 電子密度がゼロ → キャパシタンスがゼロになること
    #[test]
    fn test_solve_cv_zero_electron_density_gives_zero_capacitance() {
        let mut cv_solver = make_cv_solver(
            0.0, // mass_electron = 0 → electron_density = 0
            0.0, // donor_concentration = 0
            1.0, // barrier_height
            0.5, // ec_ef_bottom
            0.0, 1.0, 0.1, 0.02,
        );

        let capacitance = cv_solver.solve_cv(0.5);
        assert!(
            relative_eq!(capacitance, 0.0, epsilon = 1e-30),
            "capacitance should be zero with zero electron density: {}",
            capacitance
        );
    }

    /// 高いゲート電圧でキャパシタンスが非負であること
    #[test]
    fn test_solve_cv_positive_gate_voltage_gives_non_negative_capacitance() {
        let mut cv_solver = make_cv_solver(
            0.2,  // mass_electron
            1e22, // donor_concentration
            1.0,  // barrier_height
            0.1,  // ec_ef_bottom
            0.0, 5.0, 0.1, 0.02,
        );

        let capacitance = cv_solver.solve_cv(3.0);
        assert!(
            capacitance >= 0.0,
            "capacitance should be non-negative at positive gate voltage: {}",
            capacitance
        );
    }

    /// AC電圧が大きくても計算が正常に終了すること
    #[test]
    fn test_solve_cv_with_large_ac_voltage() {
        let mut cv_solver = make_cv_solver(
            0.0, 0.0, 1.0, 0.5, 0.0, 1.0, 0.1, 0.5, // 大きい AC 電圧
        );

        let capacitance = cv_solver.solve_cv(0.0);
        // mass_electron=0 → 電子密度ゼロ → キャパシタンスゼロ
        assert!(
            relative_eq!(capacitance, 0.0, epsilon = 1e-30),
            "capacitance with large AC should still be zero for zero mass: {}",
            capacitance
        );
    }

    // -----------------------------------------------------------------------
    // solve_cv() — キャパシタンスの物理的性質
    // -----------------------------------------------------------------------

    /// C-V 特性: depletion 領域ではキャパシタンスが小、
    /// accumulation 領域ではキャパシタンスが大きいこと
    #[test]
    fn test_solve_cv_depletion_vs_accumulation() {
        let mut cv_solver = make_cv_solver(
            0.2,  // mass_electron
            1e22, // donor_concentration
            1.0,  // barrier_height
            0.1,  // ec_ef_bottom
            -2.0, 5.0, 0.1, 0.02,
        );

        // depletion 領域 (gate voltage が小さい)
        let c_depletion = cv_solver.solve_cv(-1.0);
        // accumulation 領域 (gate voltage が大きい)
        let c_accumulation = cv_solver.solve_cv(3.0);

        // accumulation のキャパシタンスが depletion 以上であること
        assert!(
            c_accumulation >= c_depletion,
            "accumulation capacitance ({}) should be >= depletion capacitance ({})",
            c_accumulation,
            c_depletion
        );
    }

    // -----------------------------------------------------------------------
    // run()
    // -----------------------------------------------------------------------

    /// voltage step がゼロの場合、panic すること
    #[test]
    #[should_panic(expected = "voltage step cannot be zero")]
    fn test_run_panics_on_zero_step() {
        let mut cv_solver = make_cv_solver(
            0.0, 0.0, 1.0, 0.1, 0.0, 1.0, 0.0, // step = 0
            0.02,
        );
        cv_solver.run();
    }

    /// 正方向のスイープ (step > 0) が正常に終了すること
    /// mass_electron=0 で軽量テスト
    #[test]
    fn test_run_forward_sweep_completes() {
        let mut cv_solver = make_cv_solver(0.0, 0.0, 1.0, 0.5, 0.0, 0.2, 0.1, 0.02);
        // panic しなければ OK
        cv_solver.run();
    }

    /// 逆方向のスイープ (step < 0) が正常に終了すること
    #[test]
    fn test_run_reverse_sweep_completes() {
        let mut cv_solver = make_cv_solver(0.0, 0.0, 1.0, 0.5, 0.2, 0.0, -0.1, 0.02);
        // panic しなければ OK
        cv_solver.run();
    }

    /// start == end の場合、1点だけ計算されること（panic しない）
    #[test]
    fn test_run_single_point() {
        let mut cv_solver = make_cv_solver(0.0, 0.0, 1.0, 0.5, 0.5, 0.5, 0.1, 0.02);
        // start == end なので1回だけ計算して終了
        cv_solver.run();
    }
}
