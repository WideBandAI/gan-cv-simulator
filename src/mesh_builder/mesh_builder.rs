use crate::cli::configuration_builder::Configuration;

#[derive(Debug)]
pub enum IDX {
    Bulk(usize),
    Interface(usize),
}

#[derive(Debug)]
pub enum FixCharge {
    Bulk(f64),
    Interface(f64),
}
#[derive(Debug)]
pub struct MeshStructure {
    pub id: Vec<IDX>,
    pub depth: Vec<f64>,
    pub permittivity: Vec<f64>,
    pub dec: Vec<f64>,
    pub nd: Vec<f64>,
    pub end: Vec<f64>,
    pub nc: Vec<f64>,
    pub fixcharge: Vec<FixCharge>,
}

#[derive(Debug)]
pub struct MeshBuilder {
    pub mesh_structure: MeshStructure,
}

impl MeshBuilder {
    pub fn run(configuration: &Configuration) -> MeshStructure {
        let mut mesh_structure = MeshStructure {
            id: Vec::new(),
            depth: Vec::new(),
            permittivity: Vec::new(),
            dec: Vec::new(),
            nd: Vec::new(),
            end: Vec::new(),
            nc: Vec::new(),
            fixcharge: Vec::new(),
        };

        let mut current_depth = 0.0;
        for idx in 0..configuration.mesh_params.layer_id.len() {
            let mesh_length = configuration.mesh_params.length_per_layer[idx];
            let layer_thickness = configuration.mesh_params.layer_thickness[idx];
            let num_mesh_layers = (layer_thickness / mesh_length) as u32;
            for _ in 0..num_mesh_layers {
                mesh_structure.id.push(IDX::Bulk(idx));
                mesh_structure.depth.push(current_depth);
                mesh_structure
                    .permittivity
                    .push(configuration.device_structure.permittivity[idx]);
                mesh_structure
                    .dec
                    .push(configuration.device_structure.dec[idx]);
                mesh_structure
                    .nd
                    .push(configuration.device_structure.nd[idx]);
                mesh_structure
                    .end
                    .push(configuration.device_structure.end[idx]);
                mesh_structure
                    .nc
                    .push(configuration.device_structure.nc[idx]);
                mesh_structure.fixcharge.push(FixCharge::Bulk(
                    configuration.bulk_fixed_charge.charge_density[idx],
                ));
                current_depth += mesh_length;
            }
        }
        mesh_structure
    }
}
