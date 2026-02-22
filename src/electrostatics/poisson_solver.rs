use crate::constants::physics::*;
use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure, IDX};
use crate::physics_equations::donor_activation::ionized_donor_concentration;
use crate::physics_equations::electron_density::{BoltzmannApproximation, ElectronDensity};

#[derive(Debug)]
pub struct Potential {
    pub depth: Vec<f64>,
    pub potential: Vec<f64>,
}

#[derive(Debug)]
pub struct PoissonSolver {
    pub potential: Potential,
    pub mesh_structure: MeshStructure,
    pub temperature: f64,
    pub sor_relaxation_factor: f64,
    pub convergence_threshold: f64,
    pub max_iterations: usize,
    pub electron_density_model: Box<dyn ElectronDensity>,
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
        };
        Self {
            potential,
            mesh_structure,
            temperature,
            sor_relaxation_factor,
            convergence_threshold,
            max_iterations,
            electron_density_model: Box::new(BoltzmannApproximation {}),
        }
    }

    /// Setting the boundary conditions
    ///
    /// # Arguments
    ///
    /// - `&mut self` (`undefined`) - The mutable reference to the `PoissonSolver` instance.
    /// - `gate_voltage` (`f64`) - The voltage applied to the gate.
    /// - `barrier_height` (`f64`) - The barrier height at the gate, which is the energy difference between the gate material and the surface material.
    /// - `ec_ef_bottom` (`f64`) - The energy difference between the conduction band and Fermi level at the bottom of the structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = set_boundary_conditions();
    /// ```
    pub fn set_boundary_conditions(
        &mut self,
        gate_voltage: f64,
        barrier_height: f64,
        ec_ef_bottom: f64,
    ) {
        self.potential.potential[0] =
            -gate_voltage + barrier_height - self.mesh_structure.delta_conduction_band[0];
        self.potential.potential[self.mesh_structure.id.len() - 1] = ec_ef_bottom;
    }

    /// Solve poisson equation
    ///
    /// # Arguments
    ///
    /// - `&mut self` (`undefined`) - The mutable reference to the `PoissonSolver` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = solve_poisson();
    /// ```
    pub fn solve_poisson(&mut self) {
        let mut iteration = 0;
        let mut sum_delta_potential = self.solve_poisson_with_sor();
        while sum_delta_potential > self.convergence_threshold && iteration < self.max_iterations {
            sum_delta_potential = self.solve_poisson_with_sor();
            iteration += 1;
        }
        println!(
            "Iteration: {}, Sum of Delta Potential: {:e}",
            iteration, sum_delta_potential
        );
    }

    /// Get potential profile
    ///
    /// # Arguments
    ///
    /// - `&self` (`undefined`) - The immutable reference to the `PoissonSolver` instance.
    ///
    /// # Returns
    ///
    /// - `Vec<(f64, f64)>` - A vector of tuples representing the depth and potential values of the potential profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = get_potential_profile();
    /// ```
    pub fn get_potential_profile(&self) -> Vec<(f64, f64)> {
        self.potential
            .depth
            .iter()
            .zip(self.potential.potential.iter())
            .map(|(d, p)| (*d, *p))
            .collect()
    }

    fn solve_poisson_with_sor(&mut self) -> f64 {
        let mut sum_delta_potential = 0.0;
        for idx in 1..self.mesh_structure.id.len() - 1 {
            let delta_potential = match self.mesh_structure.id[idx] {
                IDX::Bulk(_) => self.solve_bulk(idx),
                IDX::Interface(_) => self.solve_interface(idx),
                IDX::Surface | IDX::Bottom => {
                    panic!("Boundary conditions should not be updated in SOR loop.")
                }
            };
            self.potential.potential[idx] += self.sor_relaxation_factor * delta_potential;
            sum_delta_potential += delta_potential.abs();
        }
        sum_delta_potential
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
            self.temperature,
        );

        let ionized_donor = ionized_donor_concentration(
            self.mesh_structure.donor_concentration[idx],
            self.temperature,
            self.potential.potential[idx] + self.mesh_structure.delta_conduction_band[idx]
                - self.mesh_structure.energy_level_donor[idx],
        );

        let rho = -Q_ELECTRON * (fixcharge_density + ionized_donor - electron_density);
        let delta_potential = (1.0 / (1.0 / upper_mesh_length + 1.0 / lower_mesh_length)
            * (self.potential.potential[idx - 1] / lower_mesh_length
                + self.potential.potential[idx + 1] / upper_mesh_length
                + ((lower_mesh_length + upper_mesh_length) / 2.0) * rho
                    / self.mesh_structure.permittivity[idx]))
            - self.potential.potential[idx];

        delta_potential
    }

    fn solve_interface(&self, idx: usize) -> f64 {
        let upper_mesh_length = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
        let lower_mesh_length = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
        let c_upper = self.mesh_structure.permittivity[idx - 1] / upper_mesh_length;
        let c_lower = self.mesh_structure.permittivity[idx] / lower_mesh_length;

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
