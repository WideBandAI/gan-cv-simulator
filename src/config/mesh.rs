use crate::config::structure::DeviceStructure;
use crate::constants::units::{M_TO_NM, MEV_TO_EV, NM_TO_M};
use crate::utils::{get_parsed_input, get_parsed_input_with_default};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MeshParams {
    pub layer_id: Vec<u32>,
    pub length_per_layer: Vec<f64>,
    pub layer_thickness: Vec<f64>,
    pub energy_step: f64,
}

/// Define mesh settings.
///
/// # Arguments
///
/// - `device_structure` (`&DeviceStructure`) - The device structure to base mesh parameters on.
///
/// # Returns
///
/// - `MeshParams` - The mesh parameters derived from the device structure.
///
/// # Examples
///
/// ```ignore
/// use crate::...;
///
/// let _ = define_mesh_params(&device_structure);
/// ```
pub fn define_mesh_params(device_structure: &DeviceStructure) -> MeshParams {
    println!("Define mesh parameters.");

    let layer_num: u32 =
        get_parsed_input_with_default("Enter the number of mesh layers. Default is 1: ", 1);
    let mut layer_id: Vec<u32> = Vec::new();
    let mut length_per_layer: Vec<f64> = Vec::new();
    let mut layer_thickness: Vec<f64> = Vec::new();

    let total_thickness: f64 = device_structure.thickness.iter().sum();
    println!(
        "Total device thickness from structure: {:.1} nm",
        total_thickness * M_TO_NM
    );

    for i in 0..layer_num {
        layer_id.push(i);

        let length: f64 = get_parsed_input_with_default(
            &format!(
                "Enter the mesh length (in nm) for layer {}. Default is 0.1: ",
                i
            ),
            0.1,
        );
        length_per_layer.push(length * NM_TO_M);

        if layer_num == 1 {
            println!(
                "Only one mesh layer specified. Setting thickness to total device thickness: {:.1} nm",
                total_thickness * M_TO_NM
            );
            layer_thickness.push(total_thickness);
        } else if i == (layer_num - 1) {
            let accumulated_thickness: f64 = layer_thickness.iter().sum();
            let last_thickness = total_thickness - accumulated_thickness;
            println!(
                "Setting thickness of last layer {} to remaining thickness: {:.1} nm",
                i,
                last_thickness * M_TO_NM
            );
            layer_thickness.push(last_thickness);
        } else {
            let thickness: f64 =
                get_parsed_input(&format!("Enter the thickness (in nm) for layer {}: ", i));

            let available_thickness = total_thickness - layer_thickness.iter().sum::<f64>();
            if (thickness * NM_TO_M) > available_thickness {
                layer_thickness.push(available_thickness);
                println!(
                    "Specified thickness exceeds remaining device thickness. Setting layer {} thickness to remaining thickness: {:.1} nm",
                    i,
                    available_thickness * M_TO_NM
                );
                break;
            }

            layer_thickness.push(thickness * NM_TO_M);
        }
    }

    let energy_step: f64 =
        get_parsed_input_with_default("Enter the mesh energy step (in meV). Default is 0.1: ", 0.1)
            * MEV_TO_EV;

    MeshParams {
        layer_id,
        length_per_layer,
        layer_thickness,
        energy_step,
    }
}
