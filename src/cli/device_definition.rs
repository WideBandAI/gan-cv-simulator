use crate::cli::fixcharge::BulkFixedCharge;
use crate::cli::fixcharge::InterfaceFixedCharge;
use crate::cli::fixcharge::define_bulk_fixed_charge;
use crate::cli::fixcharge::define_interface_fixed_charge;
use crate::cli::structure::DeviceStructure;
use crate::cli::structure::define_structure;

#[derive(Debug)]
pub struct DeviceDefinition {
    pub device_structure: DeviceStructure,
    pub bulk_fixed_charge: BulkFixedCharge,
    pub interface_fixed_charge: InterfaceFixedCharge,
}

impl DeviceDefinition {
    /// Create a new `DeviceDefinition` from components.
    ///
    /// # Arguments
    ///
    /// - `device_structure` (`DeviceStructure`) - The device structure.
    /// - `bulk_fixed_charge` (`BulkFixedCharge`) - The bulk fixed charge configuration.
    /// - `interface_fixed_charge` (`InterfaceFixedCharge`) - The interface fixed charge configuration.
    ///
    /// # Returns
    ///
    /// A new `DeviceDefinition` instance.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let device_structure = define_structure();
    /// let bulk_fixed_charge = define_bulk_fixed_charge(&device_structure);
    /// let interface_fixed_charge = define_interface_fixed_charge(&device_structure);
    /// let device_def = DeviceDefinition::new(device_structure, bulk_fixed_charge, interface_fixed_charge);
    /// ```
    pub fn new(
        device_structure: DeviceStructure,
        bulk_fixed_charge: BulkFixedCharge,
        interface_fixed_charge: InterfaceFixedCharge,
    ) -> Self {
        Self {
            device_structure,
            bulk_fixed_charge,
            interface_fixed_charge,
        }
    }

    /// Create a `DeviceDefinition` with default/predefined values.
    ///
    /// This function automatically constructs a complete device definition by calling
    /// the appropriate definition functions for each component (structure, bulk fixed charge,
    /// and interface fixed charge). This is useful for creating a fully initialized device
    /// definition with sensible defaults.
    ///
    /// # Returns
    ///
    /// A new `DeviceDefinition` with predefined configuration.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let device_def = DeviceDefinition::define();
    /// ```
    pub fn define() -> Self {
        let device_structure = define_structure();
        let bulk_fixed_charge = define_bulk_fixed_charge(&device_structure);
        let interface_fixed_charge = define_interface_fixed_charge(&device_structure);
        Self::new(device_structure, bulk_fixed_charge, interface_fixed_charge)
    }
}
