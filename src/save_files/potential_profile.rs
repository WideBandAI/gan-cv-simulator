use crate::constants::physics::*;
use crate::constants::units::*;
use crate::mesh::mesh_builder::IDX;
use crate::mesh::mesh_builder::{FixChargeDensity, MeshStructure};
use crate::solvers::poisson_solver::Potential;
use std::fs;
use std::io::Write;

pub fn save_potential_profile(
    mesh_structure: &MeshStructure,
    potential_profile: &Potential,
    gate_voltage: f64,
    save_dir: &str,
    filename: &str,
) -> anyhow::Result<()> {
    super::validate_save_dir(save_dir)?;
    let save_dir_path = std::path::Path::new(save_dir);

    let filename = match std::path::Path::new(filename).file_name() {
        Some(name) if name == std::path::Path::new(filename) => name,
        _ => {
            anyhow::bail!("Invalid filename: must not contain path separators.");
        }
    };

    let potential_save_dir = save_dir_path.join("potential_profiles");
    let potential_file_path = potential_save_dir.join(filename);
    fs::create_dir_all(&potential_save_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create output directory '{}': {}. Please check permissions and try again.",
            potential_save_dir.display(),
            e
        )
    })?;

    let profile = potential_profile;

    let mut file = std::fs::File::create(&potential_file_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create potential profile file '{:?}': {}",
            filename,
            e
        )
    })?;

    writeln!(
        file,
        "Name, Depth (nm), Ec (eV), Ev (eV), ns (1/cm^3), Nd+ (1/cm^3), Nd (1/cm^3), me*, εr, fix charge (C/cm^3), fix charge (C/cm^2)"
    )?;

    // gate region (at the surface of the device)
    let gate_depth = [-200.0, 0.0];
    for &depth in &gate_depth {
        writeln!(
            file,
            "Gate, {:.3}, {:.3}, {:.3}, {:.3e}, {:.3e}, {:.3e}, {:.2}, {:.2}, {:.3e}, {:.3e}",
            depth, -gate_voltage, -gate_voltage, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        )?;
    }

    for idx in 0..profile.depth.len() {
        let layer_name = mesh_structure.name[idx].clone();
        let depth_nm = profile.depth[idx] * M_TO_NM;
        let fix_charge_bulk = match mesh_structure.fixcharge_density(idx) {
            FixChargeDensity::Bulk(q) => q * PER_M3_TO_PER_CM3, // convert from C/m^3 to C/cm^3
            _ => 0.0,
        };
        let fix_charge_interface = match mesh_structure.fixcharge_density(idx) {
            FixChargeDensity::Interface(q) => q * PER_M2_TO_PER_CM2, // convert from C/m^2 to C/cm^2
            _ => 0.0,
        };

        if matches!(mesh_structure.id[idx], IDX::Interface(_)) {
            let ec1 = profile.potential[idx] + mesh_structure.delta_conduction_band(idx - 1);
            let ev1 = ec1 - mesh_structure.bandgap_energy(idx - 1);
            let me1 = mesh_structure.mass_electron(idx - 1) / M_ELECTRON;
            let epsilon_r1 = mesh_structure.permittivity(idx - 1) / EPSILON_0;
            writeln!(
                file,
                "{}, {:.3}, {:.3}, {:.3}, {:.3e}, {:.3e}, {:.3e}, {:.2}, {:.2}, {:.3e}, {:.3e}",
                layer_name,
                depth_nm,
                ec1,
                ev1,
                0.0,
                0.0,
                0.0,
                me1,
                epsilon_r1,
                fix_charge_bulk,
                fix_charge_interface
            )?;

            let ec2 = profile.potential[idx] + mesh_structure.delta_conduction_band(idx + 1);
            let ev2 = ec2 - mesh_structure.bandgap_energy(idx + 1);
            let me2 = mesh_structure.mass_electron(idx + 1) / M_ELECTRON;
            let epsilon_r2 = mesh_structure.permittivity(idx + 1) / EPSILON_0;
            writeln!(
                file,
                "{}, {:.3}, {:.3}, {:.3}, {:.3e}, {:.3e}, {:.3e}, {:.2}, {:.2}, {:.3e}, {:.3e}",
                layer_name,
                depth_nm,
                ec2,
                ev2,
                0.0,
                0.0,
                0.0,
                me2,
                epsilon_r2,
                fix_charge_bulk,
                fix_charge_interface
            )?;
        } else {
            let ec = profile.potential[idx] + mesh_structure.delta_conduction_band(idx);
            let ev = ec - mesh_structure.bandgap_energy(idx);
            let ns = profile.electron_density[idx] * PER_M3_TO_PER_CM3; // convert
            let nd_plus = profile.ionized_donor_concentration[idx] * PER_M3_TO_PER_CM3; // convert from 1/m^3 to 1/cm^3
            let nd = mesh_structure.donor_concentration(idx) * PER_M3_TO_PER_CM3; // convert from 1/m^3 to 1/cm^3
            let me = mesh_structure.mass_electron(idx) / M_ELECTRON;
            let epsilon_r = mesh_structure.permittivity(idx) / EPSILON_0;

            writeln!(
                file,
                "{}, {:.3}, {:.3}, {:.3}, {:.3e}, {:.3e}, {:.3e}, {:.2}, {:.2}, {:.3e}, {:.3e}",
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
            )?;
        }
    }

    Ok(())
}
