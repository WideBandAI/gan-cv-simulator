pub mod structure;
use structure::define_structure;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let _structure = define_structure();
    println!("Simulation complete.");
}
