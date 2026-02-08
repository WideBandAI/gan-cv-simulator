use crate::cli::boundary_conditions::define_boundary_conditions;
use crate::cli::boundary_conditions::BoundaryConditions;
use crate::cli::fixcharge::define_bulk_fixed_charge;
use crate::cli::fixcharge::define_interface_fixed_charge;
use crate::cli::fixcharge::BulkFixedCharge;
use crate::cli::fixcharge::InterfaceFixedCharge;
use crate::cli::mesh::define_mesh_params;
use crate::cli::mesh::MeshParams;
use crate::cli::structure::define_structure;
use crate::cli::structure::DeviceStructure;

#[derive(Debug)]
pub struct ParametersDefinition {
    pub device_structure: DeviceStructure,
    pub bulk_fixed_charge: BulkFixedCharge,
    pub interface_fixed_charge: InterfaceFixedCharge,
    pub mesh_params: MeshParams,
    pub boundary_conditions: BoundaryConditions,
}

impl ParametersDefinition {
    /// Create a new `ParametersDefinition` from components.
    ///
    /// # Arguments
    ///
    /// - `device_structure` (`DeviceStructure`) - The device structure.
    /// - `bulk_fixed_charge` (`BulkFixedCharge`) - The bulk fixed charge configuration.
    /// - `interface_fixed_charge` (`InterfaceFixedCharge`) - The interface fixed charge configuration.
    ///
    /// # Returns
    ///
    /// A new `ParametersDefinition` instance.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let device_structure = define_structure();
    /// let bulk_fixed_charge = define_bulk_fixed_charge(&device_structure);
    /// let interface_fixed_charge = define_interface_fixed_charge(&device_structure);
    /// let mesh_params = define_mesh_params();
    /// let device_def = ParametersDefinition::new(device_structure, bulk_fixed_charge, interface_fixed_charge, mesh_params);
    /// ```
    pub fn new(
        device_structure: DeviceStructure,
        bulk_fixed_charge: BulkFixedCharge,
        interface_fixed_charge: InterfaceFixedCharge,
        mesh_params: MeshParams,
        boundary_conditions: BoundaryConditions,
    ) -> Self {
        Self {
            device_structure,
            bulk_fixed_charge,
            interface_fixed_charge,
            mesh_params,
            boundary_conditions,
        }
    }

    /// Create a `ParametersDefinition` with default/predefined values.
    ///
    /// This function automatically constructs a complete device definition by calling
    /// the appropriate definition functions for each component (structure, bulk fixed charge,
    /// and interface fixed charge). This is useful for creating a fully initialized device
    /// definition with sensible defaults.
    ///
    /// # Returns
    ///
    /// A new `ParametersDefinition` with predefined configuration.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let device_def = ParametersDefinition::define();
    /// ```
    pub fn define() -> Self {
        let device_structure = define_structure();
        let bulk_fixed_charge = define_bulk_fixed_charge(&device_structure);
        let interface_fixed_charge = define_interface_fixed_charge(&device_structure);
        let mesh_params = define_mesh_params(&device_structure);
        let boundary_conditions = define_boundary_conditions(&device_structure);
        Self::new(
            device_structure,
            bulk_fixed_charge,
            interface_fixed_charge,
            mesh_params,
            boundary_conditions,
        )
    }
}
