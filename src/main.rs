pub mod fixcharge;
pub mod structure;
pub mod utils;

use structure::define_structure;
use fixcharge::define_bulk_fixed_charge;

fn main() {
    println!("Starting C-V simulation with the following parameters:");
    let structure = define_structure();
    let _bulk_fixed_charge = define_bulk_fixed_charge(&structure);
    println!("Simulation complete.");
}
