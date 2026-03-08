use crate::mesh_builder::mesh_builder::{FixChargeDensity, MeshStructure};
use crate::solvers::poisson_solver::Potential;
use std::fs;
use std::io::Write;

pub fn save_potential_profile(
    mesh_structure: &MeshStructure,
    potential_profile: Potential,
    save_dir: &str,
    filename: &str,
) {
    let potential_save_dir = format!("{}/{}", save_dir, "potential_profiles");
    let potential_file_path = format!("{}/{}", potential_save_dir, filename);
    fs::create_dir_all(&potential_save_dir)
        .expect("Failed to create output directory. Please check permissions and try again.");

    let profile = potential_profile;
    let mesh_structure = mesh_structure;

    // Ensure any parent directories exist so file creation doesn't fail.
    if let Some(parent) = std::path::Path::new(&potential_file_path).parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!(
                    "Failed to create directory for potential profile '{}': {}",
                    filename, e
                );
                return;
            }
        }
    }

    let mut file = match std::fs::File::create(&potential_file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "Failed to create potential profile file '{}': {}",
                filename, e
            );
            return;
        }
    };

    if writeln!(
            file,
            "Name, Depth (nm), Ec (eV), Ev (eV), ns (1/cm^3), Nd+ (1/cm^3), Nd (1/cm^3), me (kg), ε, fix charge (C/cm^3), fix charge (C/cm^2)"
        )
        .is_err()
        {
            return;
        }

    for idx in 0..profile.depth.len() {
        let layer_name = mesh_structure.name[idx].clone();
        let depth_nm = profile.depth[idx] * 1e9;
        let ec = profile.potential[idx] + mesh_structure.delta_conduction_band[idx];
        let ev = ec - mesh_structure.bandgap_energy[idx];
        let ns = profile.electron_density[idx] * 1e-6; // convert from 1/m^3 to 1/cm^3
        let nd_plus = profile.ionized_donor_concentration[idx] * 1e-6; // convert from 1/m^3 to 1/cm^3
        let nd = mesh_structure.donor_concentration[idx] * 1e-6; // convert from 1/m^3 to 1/cm^3
        let me = mesh_structure.mass_electron[idx];
        let epsilon_r = mesh_structure.permittivity[idx];
        let fix_charge_bulk = match mesh_structure.fixcharge_density[idx] {
            FixChargeDensity::Bulk(q) => q * 1e-6, // convert from C/m^3 to C/cm^3
            _ => 0.0,
        };
        let fix_charge_interface = match mesh_structure.fixcharge_density[idx] {
            FixChargeDensity::Interface(q) => q * 1e-4, // convert from C/m^2 to C/cm^2
            _ => 0.0,
        };

        if writeln!(
            file,
            "{}, {:.3}, {:.3}, {:.3}, {:.3e}, {:.3e}, {:.3e}, {:.2e}, {:.2}, {:.3e}, {:.3e}",
            layer_name,
            depth_nm,
            ec,
            ev,
            ns,
            nd_plus,
            nd,
            me,
            epsilon_r,
            fix_charge_bulk,
            fix_charge_interface
        )
        .is_err()
        {
            return;
        }
    }
}
