use crate::config::configuration_builder::Configuration;
use crate::physics_equations::interface_states::TrapStatesType;

#[derive(Debug)]
pub enum IDX {
    Bulk(usize),
    Interface(usize),
    Surface,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub enum FixChargeDensity {
    Bulk(f64),      // Charge density (C/m^3)
    Interface(f64), // Charge density (C/m^2)
}

#[derive(Debug)]
pub enum PropertyType {
    Surface(SurfaceProperties),
    Bulk(BulkProperties),
    Interface(InterfaceProperties),
    Bottom(BottomProperties),
}

#[derive(Debug)]
pub struct SurfaceProperties {
    pub permittivity: f64,
    pub delta_conduction_band: f64,
    pub bandgap_energy: f64,
}

#[derive(Debug)]
pub struct BulkProperties {
    pub mass_electron: f64,
    pub permittivity: f64,
    pub delta_conduction_band: f64,
    pub donor_concentration: f64,
    pub energy_level_donor: f64,
    pub fixcharge_density: FixChargeDensity,
    pub bandgap_energy: f64,
}

#[derive(Debug)]
pub struct InterfaceProperties {
    pub fixcharge_density: FixChargeDensity,
    pub interface_states: InterfaceStates,
}

#[derive(Debug)]
pub enum InterfaceStates {
    Distribution(InterfaceStatesDistribution),
    None,
}

#[derive(Debug)]
pub struct InterfaceStatesDistribution {
    pub id: usize,
    pub potential: Vec<f64>,
    pub dit: Vec<TrapStatesType>,
}

#[derive(Debug)]
pub struct BottomProperties {
    pub permittivity: f64,
    pub delta_conduction_band: f64,
    pub bandgap_energy: f64,
}

/// Mesh structure
///
/// # Fields
///
/// - `id` (`Vec<IDX>`) - ID of each node in the mesh, which can be `Bulk`, `Interface`, `Surface`, or `Bottom`.
/// - `depth` (`Vec<f64>`) - Depth of each node in the mesh.
/// - `mass_electron` (`Vec<f64>`) - Effective mass of electrons in each node in the mesh.
/// - `permittivity` (`Vec<f64>`) - Permittivity of each node in the mesh.
/// - `delta_conduction_band` (`Vec<f64>`) - Energy difference of the conduction band between this layer and the bottom layer (eV).
/// - `donor_concentration` (`Vec<f64>`) - Donor concentration of each node in the mesh (m^-3).
/// - `energy_level_donor` (`Vec<f64>`) - Energy level of the donor of each node in the mesh (eV, Ec-Ed).
/// - `fixcharge_density` (`Vec<FixChargeDensity>`) - Fixed charge density of each node in the mesh.
///
/// # Examples
///
/// ```
/// use crate::mesh_builder::{MeshStructure, IDX, FixChargeDensity};
///
/// let mut s = MeshStructure::new();
/// s.add_surface_node(0.0);
/// ```
#[derive(Debug)]
pub struct MeshStructure {
    pub id: Vec<IDX>,
    pub name: Vec<String>,
    pub depth: Vec<f64>,
    pub property_type: Vec<PropertyType>,
}

impl MeshStructure {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            name: Vec::new(),
            depth: Vec::new(),
            property_type: Vec::new(),
        }
    }

    pub fn add_surface_node(&mut self, depth: f64, configuration: &Configuration) {
        self.id.push(IDX::Surface);
        self.name
            .push(configuration.device_structure.name[0].clone());
        self.depth.push(depth);
        self.property_type
            .push(PropertyType::Surface(SurfaceProperties {
                permittivity: configuration.device_structure.permittivity[0],
                delta_conduction_band: configuration.device_structure.delta_conduction_band[0],
                bandgap_energy: configuration.device_structure.bandgap_energy[0],
            }));
    }

    pub fn add_interface_node(
        &mut self,
        depth: f64,
        struct_idx: usize,
        configuration: &Configuration,
    ) {
        self.id.push(IDX::Interface(struct_idx));
        self.name.push(format!(
            "Interface_{}-{}",
            configuration.device_structure.name[struct_idx],
            configuration.device_structure.name[struct_idx + 1]
        ));
        self.depth.push(depth);

        let mut interfacestates = InterfaceStatesDistribution {
            id: struct_idx,
            potential: Vec::new(),
            dit: Vec::new(),
        };

        let mut has_states = false;
        for i in 0..configuration.continuous_interface_states.interface_id.len() {
            if configuration.continuous_interface_states.interface_id[i]
                == struct_idx.try_into().unwrap()
            {
                has_states = true;
                let digsmodel = configuration.continuous_interface_states.parameters[i];
                let mut potential = 0.0;
                loop {
                    let dit = digsmodel.continuous_states(potential).unwrap();
                    interfacestates.potential.push(potential);
                    interfacestates.dit.push(dit);
                    potential += configuration.mesh_params.energy_step;
                    if potential >= digsmodel.bandgap {
                        interfacestates.potential.push(digsmodel.bandgap);
                        interfacestates
                            .dit
                            .push(digsmodel.continuous_states(digsmodel.bandgap).unwrap());
                        break;
                    }
                }
            }
        }

        let interface_states = if has_states {
            InterfaceStates::Distribution(interfacestates)
        } else {
            InterfaceStates::None
        };

        self.property_type
            .push(PropertyType::Interface(InterfaceProperties {
                fixcharge_density: FixChargeDensity::Interface(
                    configuration.interface_fixed_charge.charge_density[struct_idx],
                ),
                interface_states,
            }));
    }

    pub fn add_bulk_node(&mut self, depth: f64, struct_idx: usize, configuration: &Configuration) {
        self.id.push(IDX::Bulk(struct_idx));
        self.name
            .push(configuration.device_structure.name[struct_idx].clone());
        self.depth.push(depth);
        self.property_type.push(PropertyType::Bulk(BulkProperties {
            mass_electron: configuration.device_structure.mass_electron[struct_idx],
            permittivity: configuration.device_structure.permittivity[struct_idx],
            delta_conduction_band: configuration.device_structure.delta_conduction_band[struct_idx],
            donor_concentration: configuration.device_structure.donor_concentration[struct_idx],
            energy_level_donor: configuration.device_structure.energy_level_donor[struct_idx],
            fixcharge_density: FixChargeDensity::Bulk(
                configuration.bulk_fixed_charge.charge_density[struct_idx],
            ),
            bandgap_energy: configuration.device_structure.bandgap_energy[struct_idx],
        }));
    }

    pub fn add_bottom_node(&mut self, depth: f64, configuration: &Configuration) {
        let struct_idx = configuration.device_structure.id.len() - 1;
        self.id.push(IDX::Bottom);
        self.name.push("Bottom".to_string());
        self.depth.push(depth);
        self.property_type
            .push(PropertyType::Bottom(BottomProperties {
                permittivity: configuration.device_structure.permittivity[struct_idx],
                delta_conduction_band: configuration.device_structure.delta_conduction_band
                    [struct_idx],
                bandgap_energy: configuration.device_structure.bandgap_energy[struct_idx],
            }));
    }

    /// Get the permittivity at the given mesh index.
    pub fn permittivity(&self, idx: usize) -> f64 {
        match &self.property_type[idx] {
            PropertyType::Surface(p) => p.permittivity,
            PropertyType::Bulk(p) => p.permittivity,
            PropertyType::Bottom(p) => p.permittivity,
            PropertyType::Interface(_) => 0.0,
        }
    }

    /// Get the delta conduction band value at the given mesh index.
    pub fn delta_conduction_band(&self, idx: usize) -> f64 {
        match &self.property_type[idx] {
            PropertyType::Surface(p) => p.delta_conduction_band,
            PropertyType::Bulk(p) => p.delta_conduction_band,
            PropertyType::Bottom(p) => p.delta_conduction_band,
            PropertyType::Interface(_) => 0.0,
        }
    }

    /// Get the bandgap energy at the given mesh index.
    pub fn bandgap_energy(&self, idx: usize) -> f64 {
        match &self.property_type[idx] {
            PropertyType::Surface(p) => p.bandgap_energy,
            PropertyType::Bulk(p) => p.bandgap_energy,
            PropertyType::Bottom(p) => p.bandgap_energy,
            PropertyType::Interface(_) => 0.0,
        }
    }

    /// Get the effective electron mass at the given mesh index.
    pub fn mass_electron(&self, idx: usize) -> f64 {
        match &self.property_type[idx] {
            PropertyType::Bulk(p) => p.mass_electron,
            _ => 0.0,
        }
    }

    /// Get the donor concentration at the given mesh index.
    pub fn donor_concentration(&self, idx: usize) -> f64 {
        match &self.property_type[idx] {
            PropertyType::Bulk(p) => p.donor_concentration,
            _ => 0.0,
        }
    }

    /// Get the donor energy level at the given mesh index.
    pub fn energy_level_donor(&self, idx: usize) -> f64 {
        match &self.property_type[idx] {
            PropertyType::Bulk(p) => p.energy_level_donor,
            _ => 0.0,
        }
    }

    /// Get the fixed charge density at the given mesh index.
    pub fn fixcharge_density(&self, idx: usize) -> FixChargeDensity {
        match &self.property_type[idx] {
            PropertyType::Bulk(p) => p.fixcharge_density,
            PropertyType::Interface(p) => p.fixcharge_density,
            _ => FixChargeDensity::Bulk(0.0),
        }
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
/// use crate::config::configuration_builder::ConfigurationBuilder;
///
/// let config = ConfigurationBuilder::from_interactive().build();
/// let mesh_structure = mesh_builder::build(&config);
/// ```
pub fn build(configuration: &Configuration) -> MeshStructure {
    let mut mesh_structure = MeshStructure::new();

    let mut current_depth = 0.0;
    let mut structure_idx = 0;
    let mut total_layer_thickness = 0.0; // Updated with each interface calculation. The total thickness of layers calculated so far.
    let mut add_mesh_layer_thickness = 0.0;

    // Surface
    mesh_structure.add_surface_node(current_depth, configuration);

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
    mesh_structure.add_bottom_node(
        configuration.device_structure.thickness.iter().sum::<f64>(),
        configuration,
    );

    mesh_structure
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::boundary_conditions::BoundaryConditions;
    use crate::config::fixcharge::{BulkFixedCharge, InterfaceFixedCharge};
    use crate::config::interface_states::{
        ContinuousInterfaceStatesConfig, DiscreteInterfaceStatesConfig,
    };
    use crate::config::measurement::{Measurement, Stress, Temperature, Time, Voltage};
    use crate::config::mesh::MeshParams;
    use crate::config::sim_settings::SimSettings;
    use crate::config::structure::{DeviceStructure, MaterialType};
    use crate::physics_equations::interface_states::{DIGSModel, DiscreteModel, DiscreteStateType};
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
                    end: 1.0,
                    step: 0.1,
                },
                ac_voltage: 0.02,
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
                mass_electron: vec![0.5; num_layers],
                permittivity: vec![10.0; num_layers],
                bandgap_energy: vec![1.0; num_layers],
                delta_conduction_band: vec![0.0; num_layers],
                donor_concentration: vec![1e22; num_layers],
                energy_level_donor: vec![0.05; num_layers],
            },
            bulk_fixed_charge: BulkFixedCharge {
                layer_id: (0..num_layers as u32).collect(),
                charge_density: vec![0.0; num_layers],
            },
            interface_fixed_charge: InterfaceFixedCharge {
                interface_id: (0..num_layers.saturating_sub(1) as u32).collect(),
                charge_density: vec![0.0; num_layers.saturating_sub(1)],
            },
            continuous_interface_states: ContinuousInterfaceStatesConfig {
                interface_id: (0..num_layers.saturating_sub(1) as u32).collect(),
                parameters: vec![
                    DIGSModel::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
                    num_layers.saturating_sub(1)
                ],
            },
            discrete_interface_states: DiscreteInterfaceStatesConfig {
                interface_id: (0..num_layers.saturating_sub(1) as u32).collect(),
                parameters: vec![
                    vec![DiscreteModel::new(
                        1.0,
                        1.0,
                        1.0,
                        DiscreteStateType::DonorLike
                    )];
                    num_layers.saturating_sub(1)
                ],
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
            sim_settings: SimSettings {
                sim_name: "test_simulation".to_string(),
                sor_relaxation_factor: 1.9,
                convergence_criterion: 1e-6,
                max_iterations: 500000,
                parallel_use: false,
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
