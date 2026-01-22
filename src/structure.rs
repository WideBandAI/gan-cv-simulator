use std::io;

#[derive(Debug)]
pub struct DeviceStructure {
    pub depth: Vec<f64>,  // meters
    pub me: Vec<f64>,  // effective mass of electron
    pub er_s: Vec<f64>,  // relative permittivity
    pub eg: Vec<f64>,  // bandgap energy in eV
    pub dec: Vec<f64>,  // delta conduction band in eV from bottom layer to current layer
    pub nd: Vec<f64>,  // donor concentration in cm^-3
    pub end: Vec<f64>,  // energy level of donor in eV (Ec-Ed)
}

pub fn define_structure() -> DeviceStructure {
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
        depth: vec![],
        me: vec![],
        er_s: vec![],
        eg: vec![],
        dec: vec![],
        nd: vec![],
        end: vec![],
    };
    
    for n in 1..(num_layers + 1) {
        println!("Enter parameters for layer {}: depth (nm), me, er_s, eg (eV), dec (cm^-3), nd (cm^-3), end (eV)", n);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let params: Vec<f64> = input
            .trim()
            .split_whitespace()
            .map(|x| x.parse().expect("Failed to parse parameter"))
            .collect();
        if params.len() != 7 {
            panic!("Expected 7 parameters, got {}", params.len());
        }
        println!("Layer {} parameters: depth: {}, me: {}, er_s: {}, eg: {}, dec: {}, nd: {}, end: {}", 
                 n, params[0], params[1], params[2], params[3], params[4], params[5], params[6]);
        
        device.depth.push(params[0]);
        device.me.push(params[1]);
        device.er_s.push(params[2]);
        device.eg.push(params[3]);
        device.dec.push(params[4]);
        device.nd.push(params[5]);
        device.end.push(params[6]);
    }
    println!("Structure definition complete.");
    println!("{:?}", device);

    device
}