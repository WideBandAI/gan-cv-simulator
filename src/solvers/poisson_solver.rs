use crate::constants::physics::*;
use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure, IDX};
use crate::physics_equations::donor_activation::DonorActivation;
use crate::physics_equations::electron_density::{BoltzmannApproximation, ElectronDensity};
use indicatif::{ProgressBar, ProgressStyle};

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
    pub damping_factor: f64,
    pub convergence_threshold: f64,
    pub max_iterations: usize,
    pub electron_density_model: Box<dyn ElectronDensity>,
    pub donor_activation_model: DonorActivation,
}

impl PoissonSolver {
    pub fn new(
        mesh_structure: MeshStructure,
        initial_potential: f64,
        temperature: f64,
        damping_factor: f64,
        convergence_threshold: f64,
        max_iterations: usize,
    ) -> Self {
        let potential = Potential {
            depth: mesh_structure.depth.clone(),
            potential: vec![initial_potential; mesh_structure.id.len()],
            electron_density: vec![0.0; mesh_structure.id.len()],
            ionized_donor_concentration: vec![0.0; mesh_structure.id.len()],
        };
        Self {
            potential,
            mesh_structure,
            temperature,
            damping_factor,
            convergence_threshold,
            max_iterations,
            electron_density_model: Box::new(BoltzmannApproximation::new(temperature)),
            donor_activation_model: DonorActivation::new(temperature),
        }
    }

    pub fn set_boundary_conditions(&mut self, surface_potential: f64, bottom_potential: f64) {
        self.potential.potential[0] =
            surface_potential - self.mesh_structure.delta_conduction_band[0];
        self.potential.potential[self.mesh_structure.id.len() - 1] = bottom_potential
            - self.mesh_structure.delta_conduction_band[self.mesh_structure.id.len() - 1];
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
        self.donor_activation_model.set_temperature(temperature);
        self.electron_density_model.set_temperature(temperature);
    }

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
            sum_delta_potential = self.solve_poisson_with_newton();

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

    fn solve_poisson_with_newton(&mut self) -> f64 {
        let n = self.mesh_structure.id.len();
        let mut a = vec![0.0; n];
        let mut b = vec![0.0; n];
        let mut c = vec![0.0; n];
        let mut d = vec![0.0; n];

        let vt = K_BOLTZMANN * self.temperature / Q_ELECTRON;
        let q_per_kbt = 1.0 / vt;

        // Boundary conditions
        b[0] = 1.0;
        d[0] = 0.0;

        for i in 1..n - 1 {
            let h_up = self.mesh_structure.depth[i] - self.mesh_structure.depth[i - 1];
            let h_low = self.mesh_structure.depth[i + 1] - self.mesh_structure.depth[i];

            match self.mesh_structure.id[i] {
                IDX::Bulk(_) => {
                    let eps = self.mesh_structure.permittivity[i];
                    let c_up = eps / h_up;
                    let c_low = eps / h_low;
                    let delta_x = (h_up + h_low) / 2.0;

                    let pot_total = self.potential.potential[i] + self.mesh_structure.delta_conduction_band[i];
                    let n_i = self.electron_density_model.electron_density(pot_total, self.mesh_structure.mass_electron[i]);
                    let nd_plus = self.donor_activation_model.ionized_donor_concentration(
                        self.mesh_structure.donor_concentration[i],
                        pot_total - self.mesh_structure.energy_level_donor[i],
                    );
                    
                    let fixcharge = match self.mesh_structure.fixcharge_density[i] {
                        FixChargeDensity::Bulk(q) => q,
                        _ => 0.0,
                    };

                    let rho = Q_ELECTRON * (nd_plus + fixcharge - n_i);
                    let f_i = c_up * (self.potential.potential[i - 1] - self.potential.potential[i])
                            + c_low * (self.potential.potential[i + 1] - self.potential.potential[i])
                            - delta_x * rho;

                    let dn_dphi = -n_i * q_per_kbt;
                    
                    let v = (pot_total - self.mesh_structure.energy_level_donor[i]) * q_per_kbt;
                    let exp_v_neg = (-v).exp();
                    let dndplus_dphi = if self.mesh_structure.donor_concentration[i] > 0.0 {
                        let factor = 2.0 * exp_v_neg / (1.0 + 2.0 * exp_v_neg);
                        nd_plus * factor * q_per_kbt
                    } else {
                        0.0
                    };

                    let drho_dphi = Q_ELECTRON * (dndplus_dphi - dn_dphi);

                    a[i] = c_up;
                    b[i] = -(c_up + c_low) - delta_x * drho_dphi;
                    c[i] = c_low;
                    d[i] = -f_i;
                }
                IDX::Interface(_) => {
                    let c_up = self.mesh_structure.permittivity[i - 1] / h_up;
                    let c_low = self.mesh_structure.permittivity[i + 1] / h_low;
                    let fixcharge = match self.mesh_structure.fixcharge_density[i] {
                        FixChargeDensity::Interface(q) => q,
                        _ => 0.0,
                    };

                    let f_i = c_up * (self.potential.potential[i - 1] - self.potential.potential[i])
                            + c_low * (self.potential.potential[i + 1] - self.potential.potential[i])
                            + Q_ELECTRON * fixcharge;

                    a[i] = c_up;
                    b[i] = -(c_up + c_low);
                    c[i] = c_low;
                    d[i] = -f_i;
                }
                _ => unreachable!(),
            }
        }

        b[n - 1] = 1.0;
        d[n - 1] = 0.0;

        let delta_phi = self.solve_tridiagonal(a, b, c, d);
        let mut max_delta = 0.0;
        
        let limit = 2.0 * vt;

        for i in 0..n {
            let mut step = delta_phi[i] * self.damping_factor;
            if step.abs() > limit {
                step = limit * step.signum();
            }
            self.potential.potential[i] += step;
            if step.abs() > max_delta {
                max_delta = step.abs();
            }
        }
        max_delta
    }
    fn solve_tridiagonal(&self, a: Vec<f64>, b: Vec<f64>, c: Vec<f64>, mut d: Vec<f64>) -> Vec<f64> {
        let n = b.len();
        let mut c_prime = vec![0.0; n];
        
        c_prime[0] = c[0] / b[0];
        d[0] = d[0] / b[0];

        for i in 1..n {
            let denom = b[i] - a[i] * c_prime[i - 1];
            let m = 1.0 / denom;
            if i < n - 1 {
                c_prime[i] = c[i] * m;
            }
            d[i] = (d[i] - a[i] * d[i - 1]) * m;
        }

        let mut x = vec![0.0; n];
        x[n - 1] = d[n - 1];
        for i in (0..n - 1).rev() {
            x[i] = d[i] - c_prime[i] * x[i + 1];
        }
        x
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
            assert!(
                relative_eq!(p, initial_potential, epsilon = 1e-15),
            );
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
        assert!(relative_eq!(solver.potential.potential[1], expected_1, max_relative = 1e-6));
        assert!(relative_eq!(solver.potential.potential[2], expected_2, max_relative = 1e-6));
    }
}
