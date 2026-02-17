use crate::mesh_builder::mesh_builder::MeshStructure;

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
            gate_voltage + barrier_height - self.mesh_structure.delta_conduction_band[0];
        self.potential.potential[self.mesh_structure.id.len() - 1] = ec_ef_bottom;
    }

    pub fn solve(&self) {}

    pub fn solve_bulk(&self) {}

    pub fn solve_interface(&self) {}
}
