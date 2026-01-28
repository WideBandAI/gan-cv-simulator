use std::{io, str::FromStr, vec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialType {
    Semiconductor,
    Insulator,
}

#[derive(Debug)]
pub struct DeviceStructure {
    pub id: Vec<u32>,      // Optional: layer ID
    pub name: Vec<String>, // Optional: name of the device structure
    pub material_type: Vec<MaterialType>,
    pub thickness: Vec<f64>, // meters
    pub me: Vec<f64>,        // effective mass of electron
    pub er: Vec<f64>,        // relative permittivity
    pub eg: Vec<f64>,        // bandgap energy in eV
    pub dec: Vec<f64>,       // delta conduction band in eV from bottom layer to current layer
    pub nd: Vec<f64>,        // donor concentration in cm^-3
    pub end: Vec<f64>,       // energy level of donor in eV (Ec-Ed)
}

fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input
}

fn get_parsed_input<T: FromStr>(prompt: &str) -> T {
    loop {
        let input = get_input(prompt);
        match input.trim().parse::<T>() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please enter a valid value."),
        }
    }
}

fn get_material_type(prompt: &str) -> MaterialType {
    loop {
        let input = get_input(prompt);
        match input.trim().to_lowercase().as_str() {
            "s" => return MaterialType::Semiconductor,
            "i" => return MaterialType::Insulator,
            _ => println!("Invalid input. Please enter 's' or 'i'."),
        }
    }
}

pub fn define_structure() -> DeviceStructure {
    // Interactive structure definition
    println!("Define the structure.");
    let num_layers: u32 = get_parsed_input("Enter the number of layers: ");
    println!("Number of layers: {}", num_layers);

    let mut device = DeviceStructure {
        id: vec![],
        name: vec![],
        material_type: vec![],
        thickness: vec![],
        me: vec![],
        er: vec![],
        eg: vec![],
        dec: vec![],
        nd: vec![],
        end: vec![],
    };

    for n in 0..(num_layers) {
        device.id.push(n);
        println!("\nLayer {}:", n);

        let name = get_input(&format!(
            "Enter name for layer {} (or press Enter to skip): ",
            n
        ));
        device.name.push(name.trim().to_string());

        let mat_type = get_material_type(&format!(
            "Is layer {} a Semiconductor (s) or Insulator (i)? ",
            n
        ));
        device.material_type.push(mat_type);

        let thickness_nm: f64 = get_parsed_input(&format!("Enter thickness of layer {} (nm): ", n));
        device.thickness.push(thickness_nm * 1e-9); // convert nm to meters

        let er: f64 = get_parsed_input(&format!(
            "Enter relative permittivity (er) for layer {}: ",
            n
        ));
        device.er.push(er);

        let eg: f64 = get_parsed_input(&format!(
            "Enter bandgap energy (eg) in eV for layer {}: ",
            n
        ));
        device.eg.push(eg);

        if n == (num_layers - 1) {
            device.dec.push(0.0); // last layer delta conduction band is 0
        } else {
            let dec: f64 = get_parsed_input(&format!(
                "Enter delta conduction band (dec) in eV from bottom layer to layer {}: ",
                n
            ));
            device.dec.push(dec);
        }

        if device.material_type[n as usize] == MaterialType::Semiconductor {
            let me: f64 = get_parsed_input(&format!(
                "Enter effective mass of electron (me) for layer {}: ",
                n
            ));
            device.me.push(me);

            let nd: f64 = get_parsed_input(&format!(
                "Enter donor concentration (nd) in cm^-3 for layer {}: ",
                n
            ));
            device.nd.push(nd);

            let end: f64 = get_parsed_input(&format!(
                "Enter energy level of donor (end) in eV (Ec-Ed) for layer {}: ",
                n
            ));
            device.end.push(end);
        } else {
            device.me.push(0.0);
            device.nd.push(0.0);
            device.end.push(0.0);
        }
    }
    println!("Structure definition complete.");
    println!("{:?}", device);

    device
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_type_equality() {
        assert_eq!(MaterialType::Semiconductor, MaterialType::Semiconductor);
        assert_eq!(MaterialType::Insulator, MaterialType::Insulator);
        assert_ne!(MaterialType::Semiconductor, MaterialType::Insulator);
    }

    #[test]
    fn test_device_structure_creation() {
        let device = DeviceStructure {
            id: vec![0],
            name: vec!["test".to_string()],
            material_type: vec![MaterialType::Semiconductor],
            thickness: vec![1e-8],
            me: vec![0.5],
            er: vec![12.0],
            eg: vec![1.12],
            dec: vec![0.0],
            nd: vec![1e16],
            end: vec![0.1],
        };

        assert_eq!(device.material_type.len(), 1);
        assert_eq!(device.thickness[0], 1e-8);
        assert_eq!(device.me[0], 0.5);
        assert_eq!(device.er[0], 12.0);
    }

    #[test]
    fn test_device_structure_multiple_layers() {
        let device = DeviceStructure {
            id: vec![0, 1],
            name: vec!["layer1".to_string(), "layer2".to_string()],
            material_type: vec![MaterialType::Semiconductor, MaterialType::Insulator],
            thickness: vec![1e-8, 2e-8],
            me: vec![0.5, 0.0],
            er: vec![12.0, 3.9],
            eg: vec![1.12, 9.0],
            dec: vec![0.3, 0.0],
            nd: vec![1e16, 0.0],
            end: vec![0.1, 0.0],
        };

        assert_eq!(device.material_type.len(), 2);
        assert_eq!(device.thickness.len(), 2);
    }
}
