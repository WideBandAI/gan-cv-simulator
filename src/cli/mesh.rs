use crate::constants::units::{MEV_TO_EV, NM_TO_M};
use crate::utils::{get_parsed_input, get_parsed_input_with_default};

#[derive(Debug)]
pub struct MeshParams {
    pub layer_id: Vec<u32>,
    pub length_per_layer: Vec<f64>,
    pub layer_thickness: Vec<f64>,
    pub energy_step: f64,
}
// TODO: DeviceStructureに基づいてメッシュを定義するように修正する
// TODO: layer_numが1のときは、DeviceStructureのトータルの厚さをlength_thicknessに設定する
// TODO: layer_thicknessの合計がDeviceStructureのトータルの厚さと一致するようにする
pub fn define_mesh_params() -> MeshParams {
    println!("Define mesh parameters.");

    let layer_num: u32 =
        get_parsed_input_with_default("Enter the number of mesh layers. Default is 1: ", 1);
    let mut layer_id: Vec<u32> = Vec::new();
    let mut length_per_layer: Vec<f64> = Vec::new();
    let mut layer_thickness: Vec<f64> = Vec::new();

    for i in 0..layer_num {
        layer_id.push(i);

        let length: f64 = get_parsed_input_with_default(
            &format!("Enter the length (in nm) for layer {}. Default is 0.1: ", i),
            0.1,
        );
        length_per_layer.push(length * NM_TO_M);

        let thickness: f64 =
            get_parsed_input(&format!("Enter the thickness (in nm) for layer {}: ", i));
        layer_thickness.push(thickness * NM_TO_M);
    }

    let energy_step: f64 =
        get_parsed_input_with_default("Enter the energy step (in meV). Default is 0.1: ", 0.1)
            * MEV_TO_EV;

    MeshParams {
        layer_id,
        length_per_layer,
        layer_thickness,
        energy_step,
    }
}
