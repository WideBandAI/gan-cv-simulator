use crate::constants::physics::*;
use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure, IDX};
use crate::physics_equations::donor_activation::DonorActivation;
use crate::physics_equations::electron_density::{BoltzmannApproximation, ElectronDensity};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct Potential {
    pub depth: Vec<f64>,
    pub potential: Vec<f64>,
    pub electron_density: Vec<f64>,
    pub ionized_donor_concentration: Vec<f64>,
}

#[derive(Debug)]
pub struct PoissonSolver {
    pub potential: Potential,
    pub mesh_structure: MeshStructure,
    pub temperature: f64,
    pub sor_relaxation_factor: f64,
    pub red_indices: Vec<usize>,
    pub black_indices: Vec<usize>,
    pub convergence_threshold: f64,
    pub max_iterations: usize,
    pub electron_density_model: Box<dyn ElectronDensity>,
    pub donor_activation_model: DonorActivation,
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
            sum_delta_potential = self.solve_poisson_with_sor();

            pb.set_message(format!("Δ φ={:.3e}", sum_delta_potential));
            pb.inc(1);

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

    fn solve_poisson_with_sor(&mut self) -> f64 {
        let mut sum_delta_potential = 0.0;

        // Red phase (odd indices: 1, 3, 5, ...)
        let red_deltas: Vec<f64> = self
            .red_indices
            .par_iter()
            .map(|&idx| self.compute_delta(idx))
            .collect();
        for (i, &idx) in self.red_indices.iter().enumerate() {
            self.potential.potential[idx] += self.sor_relaxation_factor * red_deltas[i];
            sum_delta_potential += red_deltas[i].abs();
        }

        // Black phase (even indices: 2, 4, 6, ...)
        let black_deltas: Vec<f64> = self
            .black_indices
            .par_iter()
            .map(|&idx| self.compute_delta(idx))
            .collect();
        for (i, &idx) in self.black_indices.iter().enumerate() {
            self.potential.potential[idx] += self.sor_relaxation_factor * black_deltas[i];
            sum_delta_potential += black_deltas[i].abs();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure, IDX};
    use approx::relative_eq;

    fn make_simple_mesh(
        mass_electron: f64,
        permittivity: f64,
        donor_concentration: f64,
        bulk_fixcharge: f64,
    ) -> MeshStructure {
        let n = 4;
        MeshStructure {
            id: vec![IDX::Surface, IDX::Bulk(0), IDX::Bulk(0), IDX::Bottom],
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
        }
    }

    #[test]
    fn test_new_initializes_potential_with_initial_value() {
        let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
        let initial_potential = 0.5;
        let solver = PoissonSolver::new(mesh, initial_potential, 300.0, 1.0, 1e-6, 1000);

        assert_eq!(solver.potential.potential.len(), 4);
        for &p in &solver.potential.potential {
            assert!(relative_eq!(p, initial_potential, epsilon = 1e-15),);
        }
    }

    #[test]
    fn test_solve_poisson_converges_with_zero_charge() {
        let mesh = make_simple_mesh(0.0, 10.0 * EPSILON_0, 0.0, 0.0);
        let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-10, 100);
        solver.set_boundary_conditions(0.5, 0.1);

        let iters = solver.solve_poisson();
        assert!(iters <= solver.max_iterations);

        let expected_1 = 0.5 - 0.4 / 3.0;
        let expected_2 = 0.5 - 0.8 / 3.0;
        assert!(relative_eq!(
            solver.potential.potential[1],
            expected_1,
            max_relative = 1e-6
        ));
        assert!(relative_eq!(
            solver.potential.potential[2],
            expected_2,
            max_relative = 1e-6
        ));
    }
}
