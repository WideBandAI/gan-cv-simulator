use crate::constants::units::NM_TO_M;
use crate::utils::get_parsed_input;

#[derive(Debug)]
pub struct MeshParams {
    pub layer_id: Vec<u32>,
    pub length_per_layer: Vec<f64>,
    pub layer_thickness: Vec<f64>,
    pub energy_step: f64,
}

pub fn define_mesh_params() -> MeshParams {
    println!("Define mesh parameters.");

    let layer_num: u32 = get_parsed_input("Enter the number of mesh layers: ");
    let mut layer_id: Vec<u32> = Vec::new();
    let mut length_per_layer: Vec<f64> = Vec::new();
    let mut layer_thickness: Vec<f64> = Vec::new();

    for i in 0..layer_num {
        let id: u32 = get_parsed_input(&format!("Enter the ID for layer {}: ", i + 1));
        layer_id.push(id);

        let length: f64 =
            get_parsed_input(&format!("Enter the length (in nm) for layer {}: ", i + 1));
        length_per_layer.push(length * NM_TO_M);

        let thickness: f64 = get_parsed_input(&format!(
            "Enter the thickness (in nm) for layer {}: ",
            i + 1
        ));
        layer_thickness.push(thickness * NM_TO_M);
    }

    let energy_step: f64 = get_parsed_input("Enter the energy step (in eV): ");

    MeshParams {
        layer_id,
        length_per_layer,
        layer_thickness,
        energy_step,
    }
}
