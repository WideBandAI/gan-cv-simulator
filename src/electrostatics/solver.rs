use crate::mesh_builder::mesh_builder::MeshStructure;

pub struct Potential {
    pub potential: Vec<f64>,
}

pub struct Solver {
    pub potential: Potential,
}

impl Solver {
    pub fn new(mesh_structure: &MeshStructure, initial_potential: f64) -> Self {
        let potential = Potential {
            potential: vec![initial_potential; mesh_structure.id.len()],
        };
        Self { potential }
    }

    pub fn set_boundary_conditions(&self) {}

    pub fn solve(&self) {}

    pub fn solve_bulk(&self) {}

    pub fn solve_interface(&self) {}
}
