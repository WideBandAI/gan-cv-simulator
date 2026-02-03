pub mod constants;
pub mod fixcharge;
pub mod structure;
pub mod utils;

use fixcharge::{define_bulk_fixed_charge, define_interface_fixed_charge};
use structure::define_structure;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let structure = define_structure();
    let _bulk_fixed_charge = define_bulk_fixed_charge(&structure);
    let _interface_fixed_charge = define_interface_fixed_charge(&structure);
    println!("Simulation complete.");
}
