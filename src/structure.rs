use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialType {
    Semiconductor,
    Insulator,
}

#[derive(Debug)]
pub struct DeviceStructure {
    pub material_type: Vec<MaterialType>,
    pub thickness: Vec<f64>, // meters
    pub me: Vec<f64>,        // effective mass of electron
    pub er: Vec<f64>,        // relative permittivity
    pub eg: Vec<f64>,        // bandgap energy in eV
    pub dec: Vec<f64>,       // delta conduction band in eV from bottom layer to current layer
    pub nd: Vec<f64>,        // donor concentration in cm^-3
    pub end: Vec<f64>,       // energy level of donor in eV (Ec-Ed)
}

pub fn define_structure() -> DeviceStructure {
    // Interactive structure definition
    println!("Define the structure. Enter the number of layers.");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let num_layers: u32 = input
        .trim()
        .parse()
        .expect("Failed to parse number of layers");
    println!("Number of layers: {}", num_layers);

    let mut device = DeviceStructure {
        material_type: vec![],
        thickness: vec![],
        me: vec![],
        er: vec![],
        eg: vec![],
        dec: vec![],
        nd: vec![],
        end: vec![],
    };

    for n in 1..(num_layers + 1) {
        println!("Is layer {} a Semiconductor (s) or Insulator (i)? ", n);
        io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
        let mut mat_input = String::new();
        io::stdin()
            .read_line(&mut mat_input)
            .expect("Failed to read line");
        let mat_type = match mat_input.trim().to_lowercase().as_str() {
            "s" => MaterialType::Semiconductor,
            "i" => MaterialType::Insulator,
            _ => panic!("Invalid material type"),
        };
        device.material_type.push(mat_type);

        println!("Enter thickness of layer {} (nm): ", n);
        io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
        let mut thickness_input = String::new();
        io::stdin()
            .read_line(&mut thickness_input)
            .expect("Failed to read line");
        let thickness_nm: f64 = thickness_input
            .trim()
            .parse()
            .expect("Failed to parse thickness");
        device.thickness.push(thickness_nm * 1e-9); // convert nm to meters

        println!("Enter relative permittivity (er) for layer {}: ", n);
        io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
        let mut er_input = String::new();
        io::stdin()
            .read_line(&mut er_input)
            .expect("Failed to read line");
        let er: f64 = er_input.trim().parse().expect("Failed to parse er");
        device.er.push(er); // relative permittivity

        println!("Enter bandgap energy (eg) in eV for layer {}: ", n);
        io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
        let mut eg_input = String::new();
        io::stdin()
            .read_line(&mut eg_input)
            .expect("Failed to read line");
        let eg: f64 = eg_input.trim().parse().expect("Failed to parse eg");
        device.eg.push(eg); // bandgap energy in eV

        if n == num_layers {
            device.dec.push(0.0); // last layer delta conduction band is 0
        } else {
            println!(
                "Enter delta conduction band (dec) in eV from bottom layer to layer {}: ",
                n
            );
            io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
            let mut dec_input = String::new();
            io::stdin()
                .read_line(&mut dec_input)
                .expect("Failed to read line");
            let dec: f64 = dec_input.trim().parse().expect("Failed to parse dec");
            device.dec.push(dec); // delta conduction band in eV
        }

        if device.material_type[n as usize - 1] == MaterialType::Semiconductor {
            println!("Enter effective mass of electron (me) for layer {}: ", n);
            io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
            let mut me_input = String::new();
            io::stdin()
                .read_line(&mut me_input)
                .expect("Failed to read line");
            let me: f64 = me_input.trim().parse().expect("Failed to parse me");
            device.me.push(me); // effective mass of electron

            println!("Enter donor concentration (nd) in cm^-3 for layer {}: ", n);
            io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
            let mut nd_input = String::new();
            io::stdin()
                .read_line(&mut nd_input)
                .expect("Failed to read line");
            let nd: f64 = nd_input.trim().parse().expect("Failed to parse nd");
            device.nd.push(nd); // donor concentration in cm^-3

            println!(
                "Enter energy level of donor (end) in eV (Ec-Ed) for layer {}: ",
                n
            );
            io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
            let mut end_input = String::new();
            io::stdin()
                .read_line(&mut end_input)
                .expect("Failed to read line");
            let end: f64 = end_input.trim().parse().expect("Failed to parse end");
            device.end.push(end); // energy level of donor in eV
        } else {
            device.me.push(0.0); // No effective mass in insulator
            device.nd.push(0.0); // No donors in insulator
            device.end.push(0.0); // No donor energy level in insulator
        }
    }
    println!("Structure definition complete.");
    println!("{:?}", device);

    device
}
