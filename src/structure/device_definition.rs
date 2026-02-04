use crate::structure::fixcharge::BulkFixedCharge;
use crate::structure::fixcharge::InterfaceFixedCharge;
use crate::structure::fixcharge::define_bulk_fixed_charge;
use crate::structure::fixcharge::define_interface_fixed_charge;
use crate::structure::structure::DeviceStructure;
use crate::structure::structure::define_structure;

#[derive(Debug)]
pub struct DeviceDefinition {
    pub device_structure: DeviceStructure,
    pub bulk_fixed_charge: BulkFixedCharge,
    pub interface_fixed_charge: InterfaceFixedCharge,
}

/// Define Device structure
///
/// # Arguments
///
/// - `device_structure` (`DeviceStructure`) - The device structure to be defined.
/// - `bulk_fixed_charge` (`BulkFixedCharge`) - The bulk fixed charge to be defined.
/// - `interface_fixed_charge` (`InterfaceFixedCharge`) - The interface fixed charge to be defined.
///
/// # Returns
///
/// - `Self` - The newly defined `DeviceDefinition`.
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let _ = new();
/// ```
impl DeviceDefinition {
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
    pub fn define() -> Self {
        let device_structure = define_structure();
        let bulk_fixed_charge = define_bulk_fixed_charge(&device_structure);
        let interface_fixed_charge = define_interface_fixed_charge(&device_structure);
        Self::new(device_structure, bulk_fixed_charge, interface_fixed_charge)
    }
}
