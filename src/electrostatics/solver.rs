use crate::constants::physics::Q_ELECTRON;
use crate::mesh_builder::mesh_builder::{FixCharge, MeshStructure};

#[derive(Debug)]
pub struct Potential {
    pub potential: Vec<f64>,
}

#[derive(Debug)]
pub struct Solver {
    pub potential: Potential,
    pub mesh_structure: MeshStructure,
}

impl Solver {
    pub fn new(mesh_structure: MeshStructure, initial_potential: f64) -> Self {
        let potential = Potential {
            potential: vec![initial_potential; mesh_structure.id.len()],
        };
        Self {
            potential,
            mesh_structure,
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
            self.potential.potential[idx] += self.solve_interface(idx);
        }
    }

    pub fn solve_bulk(&mut self) {}

    pub fn solve_interface(&self, idx: usize) -> f64 {
        let upper_mesh_length = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
        let lower_mesh_length = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
        let c_upper = self.mesh_structure.permittivity[idx - 1] / upper_mesh_length;
        let c_lower = self.mesh_structure.permittivity[idx] / lower_mesh_length;

        let fixcharge_density = match self.mesh_structure.fixcharge[idx] {
            FixCharge::Interface(q) => q,
            _ => 0.0,
        };

        let delta_potential = 1.0 / (c_upper + c_lower)
            * (c_upper * self.potential.potential[idx - 1]
                + c_lower * self.potential.potential[idx + 1]
                - Q_ELECTRON * fixcharge_density)  // TODO: q_interface is in C/m^2. In future, we use interface states too.
            - self.potential.potential[idx];
        delta_potential
    }
}
