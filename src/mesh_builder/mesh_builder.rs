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
    pub fn build(configuration: &Configuration) -> MeshStructure {
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
        let mut structure_idx = 0;
        let mut total_layer_thickness = 0.0;
        for idx in 0..configuration.mesh_params.layer_id.len() {
            let mesh_length = configuration.mesh_params.length_per_layer[idx];
            let mesh_layer_thickness = configuration.mesh_params.layer_thickness[idx];
            let num_mesh_layers = (mesh_layer_thickness / mesh_length) as u32;
            for _ in 0..num_mesh_layers {
                if (current_depth + mesh_length)
                    > (total_layer_thickness
                        + configuration.device_structure.thickness[structure_idx])
                {
                    mesh_structure.id.push(IDX::Interface(structure_idx));
                    mesh_structure.depth.push(
                        total_layer_thickness
                            + configuration.device_structure.thickness[structure_idx],
                    );
                    mesh_structure
                        .permittivity
                        .push(configuration.device_structure.permittivity[structure_idx]);
                    mesh_structure
                        .dec
                        .push(configuration.device_structure.dec[structure_idx]);
                    mesh_structure.nd.push(0.0);
                    mesh_structure.end.push(0.0);
                    mesh_structure.nc.push(0.0);
                    mesh_structure.fixcharge.push(FixCharge::Interface(
                        configuration.interface_fixed_charge.charge_density[structure_idx],
                    ));
                    structure_idx += 1;
                    total_layer_thickness +=
                        configuration.device_structure.thickness[structure_idx];
                } else {
                    mesh_structure.id.push(IDX::Bulk(structure_idx));
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
                }
                current_depth += mesh_length;
            }
        }
        mesh_structure
    }
}
