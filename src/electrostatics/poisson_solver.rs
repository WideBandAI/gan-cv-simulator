use crate::constants::physics::*;
use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure, IDX};
use crate::physics_equations::donor_activation::ionized_donor_density;
use crate::physics_equations::electron_density::{BoltzmannApproximation, ElectronDensity};

#[derive(Debug)]
pub struct Potential {
    pub potential: Vec<f64>,
}

#[derive(Debug)]
pub struct PoissonSolver {
    pub potential: Potential,
    pub mesh_structure: MeshStructure,
    pub temperature: f64,
}

impl PoissonSolver {
    pub fn new(mesh_structure: MeshStructure, initial_potential: f64, temperature: f64) -> Self {
        let potential = Potential {
            potential: vec![initial_potential; mesh_structure.id.len()],
        };
        Self {
            potential,
            mesh_structure,
            temperature,
        }
    }

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

    pub fn solve(&mut self) {
        for idx in 1..self.mesh_structure.id.len() - 1 {
            let delta = match self.mesh_structure.id[idx] {
                IDX::Bulk(_) | IDX::Surface | IDX::Bottom => self.solve_bulk(idx),
                IDX::Interface(_) => self.solve_interface(idx),
            };
            self.potential.potential[idx] += delta;
        }
    }

    pub fn solve_bulk(&self, idx: usize) -> f64 {
        let upper_mesh_length = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
        let lower_mesh_length = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
        let c_upper = self.mesh_structure.permittivity[idx - 1] / upper_mesh_length;
        let c_lower = self.mesh_structure.permittivity[idx] / lower_mesh_length;

        let fixcharge_density = match self.mesh_structure.fixcharge_density[idx] {
            FixChargeDensity::Bulk(q) => q, // in C/m^3
            _ => 0.0,
        };

        let electron_density = BoltzmannApproximation {}.electron_density(
            self.potential.potential[idx] + self.mesh_structure.delta_conduction_band[idx],
            self.mesh_structure.mass_electron[idx],
            self.temperature,
        );

        let ionized_donor = ionized_donor_density(
            self.mesh_structure.donor_concentration[idx],
            self.temperature,
            self.potential.potential[idx] + self.mesh_structure.delta_conduction_band[idx]
                - self.mesh_structure.energy_level_donor[idx],
        );

        // rho = fixcharge + Q_ELECTRON * (ionized_donor - electron_density)
        let rho = fixcharge_density + Q_ELECTRON * (ionized_donor - electron_density);
        let node_charge = rho * (upper_mesh_length + lower_mesh_length) / 2.0;

        let delta_potential = (c_upper * self.potential.potential[idx - 1]
            + c_lower * self.potential.potential[idx + 1]
            - node_charge)
            / (c_upper + c_lower)
            - self.potential.potential[idx];

        delta_potential
    }

    pub fn solve_interface(&self, idx: usize) -> f64 {
        let upper_mesh_length = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
        let lower_mesh_length = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
        let c_upper = self.mesh_structure.permittivity[idx - 1] / upper_mesh_length;
        let c_lower = self.mesh_structure.permittivity[idx] / lower_mesh_length;

        let fixcharge_density = match self.mesh_structure.fixcharge_density[idx] {
            FixChargeDensity::Interface(q) => q, // in C/m^2
            _ => 0.0,
        };

        let delta_potential = (c_upper * self.potential.potential[idx - 1]
            + c_lower * self.potential.potential[idx + 1]
            - fixcharge_density)
            / (c_upper + c_lower)
            - self.potential.potential[idx];
        delta_potential
    }
}
