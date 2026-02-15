use crate::cli::configuration_builder::Configuration;

#[derive(Debug)]
pub enum IDX {
    Bulk(usize),
    Interface(usize),
    Surface,
    Bottom,
}

#[derive(Debug)]
pub enum FixCharge {
    Bulk(f64),
    Interface(f64),
}

/// Mesh structure
///
/// # Fields
///
/// - `id` (`Vec<IDX>`) - ID of each node in the mesh, which can be `Bulk`, `Interface`, `Surface`, or `Bottom`.
/// - `depth` (`Vec<f64>`) - Depth of each node in the mesh.
/// - `permittivity` (`Vec<f64>`) - Permittivity of each node in the mesh.
/// - `dec` (`Vec<f64>`) - Energy difference of the conduction band between this layer and the bottom layer (eV).
/// - `nd` (`Vec<f64>`) - Donor concentration of each node in the mesh (m^-3).
/// - `end` (`Vec<f64>`) - Energy level of the donor of each node in the mesh (eV, Ec-Ed).
/// - `nc` (`Vec<f64>`) - Effective density of states in the conduction band of each node in the mesh (m^-3).
/// - `fixcharge` (`Vec<FixCharge>`) - Fixed charge of each node in the mesh.
///
/// # Examples
///
/// ```
/// use crate::mesh_builder::{MeshStructure, IDX, FixCharge};
///
/// let mut s = MeshStructure::new();
/// s.add_surface_node(0.0);
/// ```
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

impl MeshStructure {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            depth: Vec::new(),
            permittivity: Vec::new(),
            dec: Vec::new(),
            nd: Vec::new(),
            end: Vec::new(),
            nc: Vec::new(),
            fixcharge: Vec::new(),
        }
    }

    fn push_properties(
        &mut self,
        id: IDX,
        depth: f64,
        permittivity: f64,
        dec: f64,
        nd: f64,
        end: f64,
        nc: f64,
        fixcharge: FixCharge,
    ) {
        self.id.push(id);
        self.depth.push(depth);
        self.permittivity.push(permittivity);
        self.dec.push(dec);
        self.nd.push(nd);
        self.end.push(end);
        self.nc.push(nc);
        self.fixcharge.push(fixcharge);
    }

    pub fn add_surface_node(&mut self, depth: f64) {
        self.push_properties(
            IDX::Surface,
            depth,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            FixCharge::Interface(0.0),
        );
    }

    pub fn add_interface_node(
        &mut self,
        depth: f64,
        struct_idx: usize,
        configuration: &Configuration,
    ) {
        self.push_properties(
            IDX::Interface(struct_idx),
            depth,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            FixCharge::Interface(configuration.interface_fixed_charge.charge_density[struct_idx]),
        );
    }

    pub fn add_bulk_node(&mut self, depth: f64, struct_idx: usize, configuration: &Configuration) {
        self.push_properties(
            IDX::Bulk(struct_idx),
            depth,
            configuration.device_structure.permittivity[struct_idx],
            configuration.device_structure.dec[struct_idx],
            configuration.device_structure.nd[struct_idx],
            configuration.device_structure.end[struct_idx],
            configuration.device_structure.nc[struct_idx],
            FixCharge::Bulk(configuration.bulk_fixed_charge.charge_density[struct_idx]),
        );
    }

    pub fn add_bottom_node(&mut self, depth: f64) {
        self.push_properties(
            IDX::Bottom,
            depth,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            FixCharge::Interface(0.0),
        );
    }
}

/// Build mesh structure
///
/// # Arguments
///
/// - `configuration` (`&Configuration`) - Configuration of the device.
///
/// # Returns
///
/// - `MeshStructure` - Mesh structure.
///
/// # Examples
///
/// ```
/// use crate::mesh_builder;
/// use crate::cli::configuration_builder::ConfigurationBuilder;
///
/// let config = ConfigurationBuilder::from_interactive().build();
/// let mesh_structure = mesh_builder::build(&config);
/// ```
pub fn build(configuration: &Configuration) -> MeshStructure {
    let mut mesh_structure = MeshStructure::new();

    let mut current_depth = 0.0;
    let mut structure_idx = 0;
    let mut total_layer_thickness = 0.0;
    let mut add_mesh_layer_thickness = 0.0;

    // Surface
    mesh_structure.add_surface_node(current_depth);

    for idx in 0..configuration.mesh_params.layer_id.len() {
        let mesh_length = configuration.mesh_params.length_per_layer[idx];
        let mesh_layer_thickness = configuration.mesh_params.layer_thickness[idx];
        add_mesh_layer_thickness += mesh_layer_thickness;

        if idx == 0 {
            current_depth += mesh_length;
        }
        loop {
            let next_interface_depth =
                total_layer_thickness + configuration.device_structure.thickness[structure_idx];
            if structure_idx < configuration.device_structure.id.len() - 1 // Interface between layers
                && current_depth >= next_interface_depth - f64::EPSILON
            {
                mesh_structure.add_interface_node(
                    next_interface_depth,
                    structure_idx,
                    configuration,
                );

                total_layer_thickness += configuration.device_structure.thickness[structure_idx];
                structure_idx += 1;
                current_depth = total_layer_thickness + mesh_length;
            } else if structure_idx == configuration.device_structure.id.len() - 1
                && current_depth >= add_mesh_layer_thickness - f64::EPSILON
            {
                break;
            } else {
                // Bulk
                mesh_structure.add_bulk_node(current_depth, structure_idx, configuration);
                current_depth += mesh_length;
            }
        }
    }
    // Bottom
    mesh_structure.add_bottom_node(configuration.device_structure.thickness.iter().sum::<f64>());

    mesh_structure
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::boundary_conditions::BoundaryConditions;
    use crate::cli::fixcharge::{BulkFixedCharge, InterfaceFixedCharge};
    use crate::cli::measurement::{Measurement, Stress, Temperature, Time, Voltage};
    use crate::cli::mesh::MeshParams;
    use crate::cli::structure::{DeviceStructure, MaterialType};
    use approx::relative_eq;

    fn create_dummy_configuration(
        thicknesses: Vec<f64>,
        mesh_lengths: Vec<f64>,
        mesh_layer_thicknesses: Vec<f64>,
    ) -> Configuration {
        let num_layers = thicknesses.len();
        Configuration {
            measurement: Measurement {
                temperature: Temperature { temperature: 300.0 },
                voltage: Voltage {
                    start: 0.0,
                    stop: 1.0,
                    step: 0.1,
                },
                time: Time {
                    measurement_time: 100.0,
                },
                stress: Stress {
                    stress_voltage: 0.0,
                    stress_relief_voltage: 0.0,
                    stress_relief_time: 0.0,
                },
            },
            device_structure: DeviceStructure {
                id: (0..num_layers as u32).collect(),
                name: vec!["layer".to_string(); num_layers],
                material_type: vec![MaterialType::Semiconductor; num_layers],
                thickness: thicknesses,
                me: vec![0.5; num_layers],
                permittivity: vec![10.0; num_layers],
                eg: vec![1.0; num_layers],
                dec: vec![0.0; num_layers],
                nd: vec![1e22; num_layers],
                end: vec![0.05; num_layers],
                nc: vec![1e25; num_layers],
            },
            bulk_fixed_charge: BulkFixedCharge {
                layer_id: (0..num_layers as u32).collect(),
                charge_density: vec![0.0; num_layers],
            },
            interface_fixed_charge: InterfaceFixedCharge {
                interface_id: (0..num_layers.saturating_sub(1) as u32).collect(),
                charge_density: vec![0.0; num_layers.saturating_sub(1)],
            },
            mesh_params: MeshParams {
                layer_id: (0..mesh_lengths.len() as u32).collect(),
                length_per_layer: mesh_lengths,
                layer_thickness: mesh_layer_thicknesses,
                energy_step: 0.001,
            },
            boundary_conditions: BoundaryConditions {
                barrier_height: 1.0,
                ec_ef_bottom: 0.1,
            },
        }
    }

    #[test]
    fn test_build_single_layer() {
        let config = create_dummy_configuration(
            vec![10e-9], // 10nm
            vec![2e-9],  // 2nm mesh
            vec![10e-9],
        );

        let mesh = build(&config);

        // Expected nodes:
        // 0.0nm (Surface)
        // 0.0nm (Bulk 0)
        // 2.0nm (Bulk 0)
        // 4.0nm (Bulk 0)
        // 6.0nm (Bulk 0)
        // 8.0nm (Bulk 0)
        // 10.0nm (Bottom)

        // Let's re-verify the logic in build
        // num_mesh_layers = 10e-9 / 2e-9 = 5
        // loop _ in 0..5:
        //  i=0: idx=0, struct_idx=0, depth=0.0 -> Surface (depth 0.0, struct_idx 0)
        //  i=1: idx=0, struct_idx=0, depth=2.0 -> Bulk (depth 2.0, struct_idx 0)
        //  i=2: idx=0, struct_idx=0, depth=4.0 -> Bulk (depth 4.0, struct_idx 0)
        //  i=3: idx=0, struct_idx=0, depth=6.0 -> Bulk (depth 6.0, struct_idx 0)
        //  i=4: idx=0, struct_idx=0, depth=8.0 -> Bulk (depth 8.0, struct_idx 0)
        // after loop: Bottom (depth 10.0)

        assert_eq!(mesh.id.len(), 6);

        assert!(
            matches!(mesh.id[0], IDX::Surface),
            "First node should be Surface"
        );
        assert_eq!(mesh.depth[0], 0.0);

        for i in 1..5 {
            assert!(
                matches!(mesh.id[i], IDX::Bulk(0)),
                "Node {} should be Bulk(0)",
                i
            );
            assert_eq!(mesh.depth[i], (i as f64) * 2e-9);
        }

        assert!(
            matches!(mesh.id[5], IDX::Bottom),
            "Last node should be Bottom"
        );
        assert!(relative_eq!(mesh.depth[5], 10e-9, max_relative = 1e-6));
    }

    #[test]
    fn test_build_multi_layer_interface() {
        // Layer 0: 5nm, Layer 1: 5nm. Total 10nm.
        // Mesh: 2.5nm steps.
        let config = create_dummy_configuration(vec![5e-9, 5e-9], vec![2.5e-9], vec![10e-9]);

        let mesh = build(&config);

        // idx = 0 (one mesh layer)
        // mesh_length = 2.5e-9
        // mesh_layer_thickness = 10e-9
        // num_mesh_layers = 4

        // i=0: depth=0.0 -> Surface
        // current_depth becomes 2.5e-9
        // i=1: depth=2.5 -> current_depth + mesh_length = 5.0.
        // structure_idx=0, thickness[0]=5e-9.
        // (2.5 + 2.5) > (0.0 + 5.0) is false. -> Bulk(0)
        // current_depth becomes 5.0
        // i=2: depth=5.0 -> current_depth + mesh_length = 7.5.
        // (5.0 + 2.5) > (0.0 + 5.0) is true. -> Interface(0)
        // total_layer_thickness becomes 5.0, structure_idx becomes 1
        // current_depth becomes 7.5
        // i=3: depth=7.5 -> current_depth + mesh_length = 10.0.
        // structure_idx=1 < 2-1 is false. -> Bulk(1)
        // current_depth becomes 10.0

        // Final: Bottom at 10.0

        // Expected IDs: Surface, Bulk(0), Interface(0), Bulk(1), Bottom
        assert_eq!(mesh.id.len(), 5);

        assert!(matches!(mesh.id[0], IDX::Surface), "node 0 fail");
        assert!(matches!(mesh.id[1], IDX::Bulk(0)), "node 1 fail");
        assert!(matches!(mesh.id[2], IDX::Interface(0)), "node 2 fail");
        assert!(matches!(mesh.id[3], IDX::Bulk(1)), "node 3 fail");
        assert!(matches!(mesh.id[4], IDX::Bottom), "node 4 fail");

        assert!(relative_eq!(mesh.depth[0], 0.0, max_relative = 1e-6));
        assert!(relative_eq!(mesh.depth[1], 2.5e-9, max_relative = 1e-6));
        assert!(relative_eq!(mesh.depth[2], 5.0e-9, max_relative = 1e-6));
        assert!(relative_eq!(mesh.depth[3], 7.5e-9, max_relative = 1e-6));
        assert!(relative_eq!(mesh.depth[4], 10.0e-9, max_relative = 1e-6));
    }
}
