pub mod structure;
use std::io;
use structure::DeviceStructure;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    println!("Define the structure. Enter the number of layers.");
    let num_layers: u32 = read_buffer();
    println!("Number of layers: {}", num_layers);
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
        DeviceStructure {
            depth: vec![params[0]],
            me: vec![params[1]],
            er_s: vec![params[2]],
            eg: vec![params[3]],
            dec: vec![params[4]],
            nd: vec![params[5]],
            end: vec![params[6]],
        };
    }
    println!("Structure definition complete.");
    println!("{:?}", DeviceStructure {
        depth: vec![],
        me: vec![],
        er_s: vec![],
        eg: vec![],
        dec: vec![],
        nd: vec![],
        end: vec![],
    });
}

fn read_buffer() -> u32 {
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(_) => buffer.trim().parse().expect("Failed to parse."),
        Err(e) => panic!("Failed to read line: {}", e)
    }
}