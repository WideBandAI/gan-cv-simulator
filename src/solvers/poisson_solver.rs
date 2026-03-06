use crate::constants::physics::*;
use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure, IDX};
use crate::physics_equations::donor_activation::DonorActivation;
use crate::physics_equations::electron_density::{BoltzmannApproximation, ElectronDensity};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Potential {
    pub depth: Vec<f64>,
    pub potential: Vec<f64>,
    pub electron_density: Vec<f64>,
    ionized_donor_concentration: Vec<f64>,
}

#[derive(Debug)]
pub struct PoissonSolver {
    pub potential: Potential,
    pub mesh_structure: MeshStructure,
    pub temperature: f64,
    sor_relaxation_factor: f64,
    red_indices: Vec<usize>,
    black_indices: Vec<usize>,
    convergence_threshold: f64,
    max_iterations: usize,
    electron_density_model: Box<dyn ElectronDensity>,
    donor_activation_model: DonorActivation,
    parallel_use: bool,
}

/// Poisson equation solver using Successive Over-Relaxation (SOR) method.
///
/// # Arguments
///
/// - `mesh_structure` (`MeshStructure`) - mesh structure containing depth, permittivity, charge densities, etc.
/// - `initial_potential` (`f64`) - The initial potential value for all mesh points.
/// - `temperature` (`f64`) - The temperature of the system, which affects the distribution of electrons and their energy levels.
/// - `sor_relaxation_factor` (`f64`) - The relaxation factor for the SOR method, which controls how much of the new value is used in updating the potential.
/// - `convergence_threshold` (`f64`) - The threshold for convergence, which determines when the iterative process stops.
/// - `max_iterations` (`usize`) - The maximum number of iterations allowed before stopping the iterative process.
/// - `electron_density_model` (`Box<dyn ElectronDensity>`) - The electron density model to use for calculating the electron density.
/// - `donor_activation_model` (`DonorActivation`) - The donor activation model to use for calculating the donor activation.
///
/// # Returns
///
/// - `Self` - An instance of `PoissonSolver` initialized with the provided parameters.
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let _ = new();
/// ```
impl PoissonSolver {
    pub fn new(
        mesh_structure: MeshStructure,
        initial_potential: f64,
        temperature: f64,
        sor_relaxation_factor: f64,
        convergence_threshold: f64,
        max_iterations: usize,
        parallel_use: bool,
    ) -> Self {
        let potential = Potential {
            depth: mesh_structure.depth.clone(),
            potential: vec![initial_potential; mesh_structure.id.len()],
            electron_density: vec![0.0; mesh_structure.id.len()],
            ionized_donor_concentration: vec![0.0; mesh_structure.id.len()],
        };
        let red_indices: Vec<usize> = (1..mesh_structure.id.len() - 1)
            .filter(|i| i % 2 == 1)
            .collect();
        let black_indices: Vec<usize> = (1..mesh_structure.id.len() - 1)
            .filter(|i| i % 2 == 0)
            .collect();
        Self {
            potential,
            mesh_structure,
            temperature,
            sor_relaxation_factor,
            red_indices,
            black_indices,
            convergence_threshold,
            max_iterations,
            electron_density_model: Box::new(BoltzmannApproximation::new(temperature)),
            donor_activation_model: DonorActivation::new(temperature),
            parallel_use,
        }
    }

    /// Setting the boundary conditions
    ///
    /// # Arguments
    ///
    /// - `surface_potential` (`f64`) - The potential at the surface Ec- Ef in eV (gate side).
    /// - `bottom_potential` (`f64`) - The potential at the bottom Ec- Ef in eV (barrier side).
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = set_boundary_conditions();
    /// ```
    pub fn set_boundary_conditions(&mut self, surface_potential: f64, bottom_potential: f64) {
        self.potential.potential[0] =
            surface_potential - self.mesh_structure.delta_conduction_band[0];
        self.potential.potential[self.mesh_structure.id.len() - 1] = bottom_potential
            - self.mesh_structure.delta_conduction_band[self.mesh_structure.id.len() - 1];
    }

    /// Set temperature
    ///
    /// # Arguments
    ///
    /// - `temperature` (`f64`) - The temperature of the system.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = set_temperature();
    /// ```
    pub fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
        self.donor_activation_model.set_temperature(temperature);
        self.electron_density_model.set_temperature(temperature);
    }

    /// Solve poisson equation
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = solve_poisson();
    /// ```
    pub fn solve_poisson(&mut self) -> usize {
        let pb = ProgressBar::new(self.max_iterations as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .unwrap(),
        );

        let mut sum_delta_potential = 0.0;
        let mut iter_count: usize = 0;

        for i in 1..=self.max_iterations {
            iter_count = i;
            sum_delta_potential = self.solve_poisson_with_sor(self.parallel_use);

            // update progress bar message with current sum of delta potential
            pb.set_message(format!("Δ φ={:.3e}", sum_delta_potential));
            pb.inc(1);

            // break early if convergence criterion satisfied
            if sum_delta_potential <= self.convergence_threshold {
                break;
            }
        }

        pb.set_position(iter_count as u64);
        if iter_count >= self.max_iterations {
            pb.finish_with_message(format!(
                "Δ φ={:.3e}. Reached max iterations without convergence.",
                sum_delta_potential
            ));
        } else {
            pb.abandon_with_message(format!(
                "Δ φ={:.3e}. Reached convergence criterion.",
                sum_delta_potential
            ));
        }
        iter_count
    }

    fn solve_poisson_with_sor(&mut self, parallel_use: bool) -> f64 {
        let mut sum_delta_potential = 0.0;

        if parallel_use {
            sum_delta_potential += self.solve_poisson_with_sor_parallel();
        } else {
            sum_delta_potential += self.solve_poisson_with_single_thread();
        }
        sum_delta_potential
    }

    fn solve_poisson_with_single_thread(&mut self) -> f64 {
        let mut sum_delta_potential = 0.0;
        for idx in 1..self.mesh_structure.id.len() - 1 {
            let delta_potential = self.compute_delta(idx);
            self.potential.potential[idx] += self.sor_relaxation_factor * delta_potential;
            sum_delta_potential += delta_potential.abs();
        }
        sum_delta_potential
    }

    fn solve_poisson_with_sor_parallel(&mut self) -> f64 {
        let mut sum_delta_potential = 0.0;

        // Red phase (odd indices: 1, 3, 5, ...)
        let red_deltas: Vec<f64> = self
            .red_indices
            .par_iter()
            .map(|&idx| self.compute_delta(idx))
            .collect();
        for (&idx, &delta) in self.red_indices.iter().zip(&red_deltas) {
            self.potential.potential[idx] += self.sor_relaxation_factor * delta;
            sum_delta_potential += delta.abs();
        }

        // Black phase (even indices: 2, 4, 6, ...)
        let black_deltas: Vec<f64> = self
            .black_indices
            .par_iter()
            .map(|&idx| self.compute_delta(idx))
            .collect();
        for (&idx, &delta) in self.black_indices.iter().zip(&black_deltas) {
            self.potential.potential[idx] += self.sor_relaxation_factor * delta;
            sum_delta_potential += delta.abs();
        }

        sum_delta_potential
    }

    fn compute_delta(&self, idx: usize) -> f64 {
        match self.mesh_structure.id[idx] {
            IDX::Bulk(_) => self.solve_bulk(idx),
            IDX::Interface(_) => self.solve_interface(idx),
            IDX::Surface | IDX::Bottom => {
                panic!("Boundary conditions should not be updated in SOR loop.")
            }
        }
    }

    fn solve_bulk(&self, idx: usize) -> f64 {
        let upper_mesh_length = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
        let lower_mesh_length = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
        let fixcharge_density = match self.mesh_structure.fixcharge_density[idx] {
            FixChargeDensity::Bulk(q) => q, // in 1/m^3
            _ => 0.0,
        };

        let electron_density = self.electron_density_model.electron_density(
            self.potential.potential[idx] + self.mesh_structure.delta_conduction_band[idx],
            self.mesh_structure.mass_electron[idx],
        );

        let ionized_donor = self.donor_activation_model.ionized_donor_concentration(
            self.mesh_structure.donor_concentration[idx],
            self.potential.potential[idx] + self.mesh_structure.delta_conduction_band[idx]
                - self.mesh_structure.energy_level_donor[idx],
        );

        let rho = -Q_ELECTRON * (fixcharge_density + ionized_donor - electron_density);
        let delta_potential = (1.0 / (upper_mesh_length + lower_mesh_length))
            * (lower_mesh_length * self.potential.potential[idx - 1]
                + upper_mesh_length * self.potential.potential[idx + 1])
            + (lower_mesh_length * upper_mesh_length * rho
                / (2.0 * self.mesh_structure.permittivity[idx]))
            - self.potential.potential[idx];

        delta_potential
    }

    fn solve_interface(&self, idx: usize) -> f64 {
        let upper_mesh_length = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
        let lower_mesh_length = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
        let c_upper = self.mesh_structure.permittivity[idx - 1] / upper_mesh_length;
        let c_lower = self.mesh_structure.permittivity[idx + 1] / lower_mesh_length;

        let fixcharge_density = match self.mesh_structure.fixcharge_density[idx] {
            FixChargeDensity::Interface(q) => q, // in 1/m^2
            _ => 0.0,
        };

        let delta_potential = (c_upper * self.potential.potential[idx - 1]
            + c_lower * self.potential.potential[idx + 1]
            - Q_ELECTRON * fixcharge_density)
            / (c_upper + c_lower)
            - self.potential.potential[idx];
        delta_potential
    }

    /// Get potential profile
    ///
    /// # Returns
    ///
    /// - `Vec<(f64, f64, f64, f64)>` - A vector of tuples containing depth, potential, electron density, and ionized donor concentration at each mesh point.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = get_potential_profile();
    /// ```
    pub fn get_potential_profile(&mut self) -> Potential {
        self.calculate_electron_density();
        self.calculate_ionized_donor_concentration();
        self.potential.clone()
    }

    fn calculate_electron_density(&mut self) {
        for idx in 0..self.mesh_structure.id.len() {
            self.potential.electron_density[idx] = self.electron_density_model.electron_density(
                self.potential.potential[idx] + self.mesh_structure.delta_conduction_band[idx],
                self.mesh_structure.mass_electron[idx],
            );
        }
    }

    fn calculate_ionized_donor_concentration(&mut self) {
        for idx in 0..self.mesh_structure.id.len() {
            self.potential.ionized_donor_concentration[idx] =
                self.donor_activation_model.ionized_donor_concentration(
                    self.mesh_structure.donor_concentration[idx],
                    self.potential.potential[idx] + self.mesh_structure.delta_conduction_band[idx]
                        - self.mesh_structure.energy_level_donor[idx],
                );
        }
    }

    pub fn save_potential_profile(&mut self, filename: &str) {
        let profile = self.get_potential_profile();
        let mesh_structure = &self.mesh_structure;
        let mut file = std::fs::File::create(filename).unwrap();
        writeln!(
            file,
            "Name, Depth (nm), Ec (eV), Ev (eV), ns (1/cm^3), Nd+ (1/cm^3), Nd (1/cm^3), me (kg), ε, fix charge (C/cm^3), fix charge (C/cm^2)"
        )
        .unwrap();
        for idx in 0..profile.depth.len() {
            let layer_name = mesh_structure.name[idx].clone();
            let depth_nm = profile.depth[idx] * 1e9;
            let ec = profile.potential[idx] + mesh_structure.delta_conduction_band[idx];
            let ev = ec - mesh_structure.bandgap_energy[idx];
            let ns = profile.electron_density[idx] * 1e-6; // convert from 1/m^3 to 1/cm^3
            let nd_plus = profile.ionized_donor_concentration[idx] * 1e-6; // convert from 1/m^3 to 1/cm^3
            let nd = mesh_structure.donor_concentration[idx] * 1e-6; // convert from 1/m^3 to 1/cm^3
            let me = mesh_structure.mass_electron[idx];
            let epsilon_r = mesh_structure.permittivity[idx];
            let fix_charge_bulk = match mesh_structure.fixcharge_density[idx] {
                FixChargeDensity::Bulk(q) => q * 1e-6, // convert from C/m^3 to C/cm^3
                _ => 0.0,
            };
            let fix_charge_interface = match mesh_structure.fixcharge_density[idx] {
                FixChargeDensity::Interface(q) => q * 1e-4, // convert from C/m^2 to C/cm^2
                _ => 0.0,
            };
            writeln!(
                file,
                "{}, {:.3}, {:.3}, {:.3}, {:.3e}, {:.3e}, {:.3e}, {:.2e}, {:.2}, {:.3e}, {:.3e}",
                layer_name,
                depth_nm,
                ec,
                ev,
                ns,
                nd_plus,
                nd,
                me,
                epsilon_r,
                fix_charge_bulk,
                fix_charge_interface
            )
            .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure, IDX};
    use approx::relative_eq;

    // -----------------------------------------------------------------------
    // Helper: 最小限の MeshStructure を手動で組み立てる
    //
    // ノード構成:
    //   [0] Surface    depth=0.0
    //   [1] Bulk(0)    depth=1e-9
    //   [2] Bulk(0)    depth=2e-9
    //   [3] Bottom     depth=3e-9
    // -----------------------------------------------------------------------
    fn make_simple_mesh(
        mass_electron: f64,
        permittivity: f64,
        donor_concentration: f64,
        bulk_fixcharge: f64,
    ) -> MeshStructure {
        let n = 4;
        MeshStructure {
            id: vec![IDX::Surface, IDX::Bulk(0), IDX::Bulk(0), IDX::Bottom],
            name: vec![
                "Surface".to_string(),
                "Bulk".to_string(),
                "Bulk".to_string(),
                "Bottom".to_string(),
            ],
            depth: vec![0.0, 1e-9, 2e-9, 3e-9],
            mass_electron: vec![0.0, mass_electron, mass_electron, 0.0],
            permittivity: vec![0.0, permittivity, permittivity, 0.0],
            delta_conduction_band: vec![0.0; n],
            donor_concentration: vec![0.0, donor_concentration, donor_concentration, 0.0],
            energy_level_donor: vec![0.0, 0.05, 0.05, 0.0],
            fixcharge_density: vec![
                FixChargeDensity::Bulk(0.0),
                FixChargeDensity::Bulk(bulk_fixcharge),
                FixChargeDensity::Bulk(bulk_fixcharge),
                FixChargeDensity::Bulk(0.0),
            ],
            bandgap_energy: vec![1.12; n],
        }
    }

    // ノード構成 (Interface 含む):
    //   [0] Surface      depth=0.0
    //   [1] Bulk(0)      depth=1
    //   [2] Interface(0) depth=2
    //   [3] Bulk(1)      depth=3
    //   [4] Bottom       depth=4
    // -----------------------------------------------------------------------
    fn make_simple_insulator_mesh(permittivity: f64, bulk_fixcharge: f64) -> MeshStructure {
        let n = 4;
        MeshStructure {
            id: vec![IDX::Surface, IDX::Bulk(0), IDX::Bulk(0), IDX::Bottom],
            name: vec![
                "Surface".to_string(),
                "Bulk".to_string(),
                "Bulk".to_string(),
                "Bottom".to_string(),
            ],
            depth: vec![0.0, 1.0, 2.0, 3.0],
            mass_electron: vec![0.0, 0.0, 0.0, 0.0],
            permittivity: vec![0.0, permittivity, permittivity, 0.0],
            delta_conduction_band: vec![0.0; n],
            donor_concentration: vec![0.0, 0.0, 0.0, 0.0],
            energy_level_donor: vec![0.0, 0.0, 0.0, 0.0],
            fixcharge_density: vec![
                FixChargeDensity::Bulk(0.0),
                FixChargeDensity::Bulk(bulk_fixcharge),
                FixChargeDensity::Bulk(bulk_fixcharge),
                FixChargeDensity::Bulk(0.0),
            ],
            bandgap_energy: vec![1.12; n],
        }
    }

    // ノード構成 (Interface 含む):
    //   [0] Surface      depth=0.0
    //   [1] Bulk(0)      depth=1e-9
    //   [2] Interface(0) depth=2e-9
    //   [3] Bulk(1)      depth=3e-9
    //   [4] Bottom       depth=4e-9
    fn make_interface_mesh(permittivity: f64, interface_fixcharge: f64) -> MeshStructure {
        let n = 5;
        MeshStructure {
            id: vec![
                IDX::Surface,
                IDX::Bulk(0),
                IDX::Interface(0),
                IDX::Bulk(1),
                IDX::Bottom,
            ],
            name: vec![
                "Surface".to_string(),
                "Bulk".to_string(),
                "Interface".to_string(),
                "Bulk".to_string(),
                "Bottom".to_string(),
            ],
            depth: vec![0.0, 1.0, 2.0, 3.0, 4.0],
            mass_electron: vec![0.0, 0.2, 0.0, 0.2, 0.0],
            permittivity: vec![0.0, permittivity, 0.0, permittivity, 0.0],
            delta_conduction_band: vec![0.0; n],
            donor_concentration: vec![0.0, 1e22, 0.0, 1e22, 0.0],
            energy_level_donor: vec![0.0, 0.05, 0.0, 0.05, 0.0],
            fixcharge_density: vec![
                FixChargeDensity::Bulk(0.0),
                FixChargeDensity::Bulk(0.0),
                FixChargeDensity::Interface(interface_fixcharge),
                FixChargeDensity::Bulk(0.0),
                FixChargeDensity::Bulk(0.0),
            ],
            bandgap_energy: vec![1.12; n],
        }
    }

    // -----------------------------------------------------------------------
    // new()
    // -----------------------------------------------------------------------

    /// new() を呼んだとき、potential が initial_potential で初期化されること
    #[test]
    fn test_new_initializes_potential_with_initial_value() {
        let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
        let initial_potential = 0.5;
        let solver = PoissonSolver::new(mesh, initial_potential, 300.0, 1.0, 1e-6, 1000, false);

        assert_eq!(solver.potential.potential.len(), 4);
        for &p in &solver.potential.potential {
            assert!(
                relative_eq!(p, initial_potential, epsilon = 1e-15),
                "initial potential mismatch: {} != {}",
                p,
                initial_potential
            );
        }
    }

    /// new() の depth が mesh_structure.depth と一致すること
    #[test]
    fn test_new_copies_depth_from_mesh() {
        let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
        let expected_depth = mesh.depth.clone();
        let solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false);

        assert_eq!(solver.potential.depth, expected_depth);
    }

    // -----------------------------------------------------------------------
    // set_boundary_conditions()
    // -----------------------------------------------------------------------

    /// 表面ポテンシャル = -Vg + barrier_height - delta_Ec[0]
    #[test]
    fn test_set_boundary_conditions_surface() {
        let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
        let delta_ec_0 = mesh.delta_conduction_band[0]; // 0.0
        let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false);

        let gate_voltage = 1.0;
        let barrier_height = 0.8;
        let ec_ef_bottom = 0.1;
        solver.set_boundary_conditions(-gate_voltage + barrier_height, ec_ef_bottom);

        let expected_surface = -gate_voltage + barrier_height - delta_ec_0;
        assert!(
            relative_eq!(
                solver.potential.potential[0],
                expected_surface,
                epsilon = 1e-15
            ),
            "surface potential: {} != {}",
            solver.potential.potential[0],
            expected_surface
        );
    }

    /// 底面ポテンシャル = ec_ef_bottom
    #[test]
    fn test_set_boundary_conditions_bottom() {
        let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
        let n = mesh.id.len();
        let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false);

        let ec_ef_bottom = 0.3;
        solver.set_boundary_conditions(0.0, ec_ef_bottom);

        assert!(
            relative_eq!(
                solver.potential.potential[n - 1],
                ec_ef_bottom,
                epsilon = 1e-15
            ),
            "bottom potential: {} != {}",
            solver.potential.potential[n - 1],
            ec_ef_bottom
        );
    }

    // -----------------------------------------------------------------------
    // solve_poisson() — 収束テスト
    // -----------------------------------------------------------------------

    /// ドナー濃度・固定電荷がゼロの場合、収束後のポテンシャルは線形補間になること
    ///
    /// mass_electron=0.0 とすることで有効状態密度 Nc=0 → electron_density=0 が保証される。
    #[test]
    fn test_solve_poisson_converges_with_zero_charge() {
        // mass_electron=0 → Nc=0 → electron_density=0
        // donor_concentration=0 → ionized_donor=0
        // fixcharge=0 → rho=0 完全にゼロ電荷
        let mesh = make_simple_mesh(0.0, 10.0 * EPSILON_0, 0.0, 0.0);
        let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-10, 100_000, false);
        solver.set_boundary_conditions(0.5, 0.1); // surface=0.5, bottom=0.1

        let iters = solver.solve_poisson(); // panic しないこと
        assert!(
            iters <= solver.max_iterations,
            "iterations should not exceed max"
        );

        // 境界条件が変わっていないこと
        assert!(
            relative_eq!(solver.potential.potential[0], 0.5, epsilon = 1e-12),
            "surface BC changed"
        );
        assert!(
            relative_eq!(solver.potential.potential[3], 0.1, epsilon = 1e-12),
            "bottom BC changed"
        );

        // 電荷ゼロ・均一メッシュ → 内部は線形補間に収束
        // depth: 0, 1, 2, 3 nm → potential: 0.5, 0.5-0.4/3, 0.5-0.8/3, 0.1
        let expected_1 = 0.5 - 0.4 / 3.0;
        let expected_2 = 0.5 - 0.8 / 3.0;
        assert!(
            relative_eq!(
                solver.potential.potential[1],
                expected_1,
                max_relative = 1e-4
            ),
            "potential[1] = {} (expected {})",
            solver.potential.potential[1],
            expected_1
        );
        assert!(
            relative_eq!(
                solver.potential.potential[2],
                expected_2,
                max_relative = 1e-4
            ),
            "potential[2] = {} (expected {})",
            solver.potential.potential[2],
            expected_2
        );
    }

    /// 閾値を非常に大きくすると、1回目のイテレーションで収束判定が
    /// 真となり返り値が 1 になること
    #[test]
    fn test_solve_poisson_returns_one_iteration_if_threshold_large() {
        let mesh = make_simple_mesh(0.0, 10.0 * EPSILON_0, 0.0, 0.0);
        let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, f64::MAX, 1000, false);
        solver.set_boundary_conditions(0.0, 0.2);

        let iters = solver.solve_poisson();
        assert_eq!(
            iters, 1,
            "solver should stop after first iteration with huge threshold"
        );
    }

    /// 負の閾値を与えると収束判定が絶対に成立せず、
    /// `max_iterations` 全部が実行されること
    #[test]
    fn test_solve_poisson_runs_full_iterations_if_threshold_negative() {
        let mesh = make_simple_mesh(0.0, 10.0 * EPSILON_0, 0.0, 0.0);
        let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, -1.0, 123, false);
        solver.set_boundary_conditions(0.0, 0.5);

        let iters = solver.solve_poisson();
        assert_eq!(iters, solver.max_iterations);
    }

    // -----------------------------------------------------------------------
    // solve_interface() — インターフェースノードの delta_potential
    // -----------------------------------------------------------------------

    /// 固定電荷ゼロのとき、solve_interface の delta_potential を検証する。
    ///
    /// solve_interface の実装では:
    ///   c_upper = permittivity[idx-1] / upper_mesh_length  (上の Bulk ノードの誘電率)
    ///   c_lower = permittivity[idx]   / lower_mesh_length  (Interface ノード自身の誘電率 = 0)
    /// Interface ノードの permittivity は 0.0 なので c_lower = 0 となる。
    /// このとき delta = c_upper * potential[idx-1] / c_upper − potential[idx]
    ///              = potential[idx-1] − potential[idx] = 0.2 − 0.0 = 0.2
    #[test]
    fn test_solve_interface_zero_fixcharge_gives_average() {
        let eps = 10.0 * EPSILON_0;
        let mesh = make_interface_mesh(eps, 0.0);

        // potential: Surface=0.0, Bulk=0.2, Interface=0.0, Bulk=0.4, Bottom=0.0
        let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1, false);
        solver.potential.potential[0] = 0.0;
        solver.potential.potential[1] = 0.2;
        solver.potential.potential[2] = 0.0; // interface の現在値
        solver.potential.potential[3] = 0.4;
        solver.potential.potential[4] = 0.0;

        // c_lower = permittivity[2] / lower_mesh_length = 0 / 1e-9 = 0
        // → delta = potential[1] − potential[2] = 0.2 − 0.0 = 0.2
        let delta = solver.solve_interface(2);
        assert!(
            relative_eq!(delta, 0.3, epsilon = 1e-12),
            "interface delta_potential = {} (expected 0.3)",
            delta
        );
    }

    /// 正の固定電荷があるとき、delta_potential が電荷なしより小さくなること
    #[test]
    fn test_solve_interface_positive_fixcharge_reduces_potential() {
        let eps = 10.0 * EPSILON_0;
        let mesh_no_charge = make_interface_mesh(eps, 0.0);
        let mesh_with_charge = make_interface_mesh(eps, 1e12); // 1e12 C/m^2

        let set_potentials = |s: &mut PoissonSolver| {
            s.potential.potential[0] = 0.0;
            s.potential.potential[1] = 0.2;
            s.potential.potential[2] = 0.0;
            s.potential.potential[3] = 0.4;
            s.potential.potential[4] = 0.0;
        };

        let mut s0 = PoissonSolver::new(mesh_no_charge, 0.0, 300.0, 1.0, 1e-6, 1, false);
        set_potentials(&mut s0);
        let delta_no_charge = s0.solve_interface(2);

        let mut s1 = PoissonSolver::new(mesh_with_charge, 0.0, 300.0, 1.0, 1e-6, 1, false);
        set_potentials(&mut s1);
        let delta_with_charge = s1.solve_interface(2);

        // Q_ELECTRON * q > 0 → 分子が小さくなる → delta が小さい
        assert!(
            delta_with_charge < delta_no_charge,
            "positive fixcharge should reduce delta_potential: {} vs {}",
            delta_with_charge,
            delta_no_charge
        );
    }

    // -----------------------------------------------------------------------
    // solve_bulk() — バルクノードの delta_potential
    // -----------------------------------------------------------------------

    /// 均一ポテンシャル・ゼロ電荷では delta_potential ≒ 0 になること
    #[test]
    fn test_solve_bulk_uniform_potential_gives_zero_delta() {
        let eps = 10.0 * EPSILON_0;
        let uniform_pot = 5.0; // 高いポテンシャル → electron_density ≈ 0
        let mesh = make_simple_mesh(0.2, eps, 0.0, 0.0);

        let mut solver = PoissonSolver::new(mesh, uniform_pot, 300.0, 1.0, 1e-6, 1, false);
        // 全ノードを同じ値に揃える
        for p in solver.potential.potential.iter_mut() {
            *p = uniform_pot;
        }

        let delta = solver.solve_bulk(1);

        assert!(
            delta.abs() < 1e-6,
            "uniform potential should give ~0 delta: {}",
            delta
        );
    }

    /// 等間隔メッシュ・ゼロ電荷では、solve_bulk の delta ≒ 両隣平均 − 現在値 になること
    #[test]
    fn test_solve_bulk_zero_charge_approaches_average() {
        let eps = 10.0 * EPSILON_0;
        // donor_concentration=0 → ionized_donor ≈ 0, electron_density ≈ 0 (高ポテンシャル时)
        let mesh = make_simple_mesh(0.2, eps, 0.0, 0.0);

        let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1, false);
        solver.potential.potential[0] = 0.0;
        solver.potential.potential[1] = 5.0; // 高いポテンシャル → rho≈0
        solver.potential.potential[2] = 1.0;
        solver.potential.potential[3] = 0.0;

        // rho≈0 のとき delta ≈ (0.0 + 1.0)/2 − 5.0 = -4.5
        let delta = solver.solve_bulk(1);
        assert!(
            relative_eq!(delta, -4.5, max_relative = 1e-4),
            "bulk delta should approach average: {} (expected -4.5)",
            delta
        );
    }

    /// bulk potentialの更新
    #[test]
    fn test_solve_bulk_with_charge() {
        let eps = 1.0;
        let bulk_fixcharge = 1.0 / Q_ELECTRON;
        let mesh = make_simple_insulator_mesh(eps, bulk_fixcharge);
        let initial_potential = 0.0;
        let solver = PoissonSolver::new(mesh, initial_potential, 300.0, 1.0, 1e-6, 1000, false);
        let delta_poisson = solver.solve_bulk(1);

        assert!(
            relative_eq!(delta_poisson, -0.5, max_relative = 1e-4),
            "bulk delta should approach average: {} (expected -0.5)",
            delta_poisson
        );
    }

    //interface potentialの更新
    #[test]
    fn test_solve_interface_with_charge() {
        let eps = 1.0;
        let interface_fixcharge = 1.0 / Q_ELECTRON;
        let mesh = make_interface_mesh(eps, interface_fixcharge);
        let initial_potential = 1.0;
        let solver = PoissonSolver::new(mesh, initial_potential, 300.0, 1.0, 1e-6, 1000, false);
        let delta_poisson = solver.solve_interface(2);

        assert!(
            relative_eq!(delta_poisson, -0.5, max_relative = 1e-6),
            "interface delta should be affected by fixcharge: {} (expected -0.5)",
            delta_poisson
        );
    }
}
