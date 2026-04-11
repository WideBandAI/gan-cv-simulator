use crate::constants::units::{M2_TO_CM2, PER_M2_TO_PER_CM2};
use crate::mesh::mesh_builder::{IDX, InterfaceStates, MeshStructure};
use std::fs;
use std::io::Write;

pub fn save_interface_states(
    mesh_structure: &MeshStructure,
    previous_phase_occupation: &[Option<Vec<f64>>],
    gate_voltage: f64,
    save_dir: &str,
    index: usize,
) -> anyhow::Result<()> {
    super::validate_save_dir(save_dir)?;

    let save_dir_path = std::path::Path::new(save_dir);
    let interface_states_dir = save_dir_path.join("interface_states");

    let mut dir_created = false;

    for (idx, id) in mesh_structure.id.iter().enumerate() {
        if !matches!(id, IDX::Interface(_)) {
            continue;
        }

        let dist = match mesh_structure.interface_states(idx) {
            Some(InterfaceStates::Distribution(d)) => d,
            _ => continue,
        };

        let occ = match previous_phase_occupation[idx].as_ref() {
            Some(o) => o,
            None => continue,
        };

        if dist.potential.is_empty() {
            continue;
        }

        if !dir_created {
            fs::create_dir_all(&interface_states_dir).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create output directory '{}': {}. Please check permissions and try again.",
                    interface_states_dir.display(),
                    e
                )
            })?;
            dir_created = true;
        }

        let filename = format!(
            "{}_{}_interfacestates_{:.3}V.csv",
            index, dist.id, gate_voltage
        );
        let file_path = interface_states_dir.join(&filename);

        let mut file = fs::File::create(&file_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create interface states file '{}': {}",
                filename,
                e
            )
        })?;

        writeln!(
            file,
            "Name, Ec-E(eV), acceptor_like_dit (cm^-2 eV^-1), donor_like_dit (cm^-2 eV^-1), occupation_probability, qit (cm^-2 eV^-1), capture_cross_section (cm^2)"
        )?;

        let layer_name = &mesh_structure.name[idx];
        for (k, &ec_e) in dist.potential.iter().enumerate() {
            let acceptor_dit = dist.acceptor_dit[k] * PER_M2_TO_PER_CM2;
            let donor_dit = dist.donor_dit[k] * PER_M2_TO_PER_CM2;
            let f = occ[k];
            let qit =
                (-dist.acceptor_dit[k] * f + dist.donor_dit[k] * (1.0 - f)) * PER_M2_TO_PER_CM2;
            let capture_cross_section_value = dist.capture_cross_section[k] * M2_TO_CM2;

            writeln!(
                file,
                "{}, {:.6}, {:.6e}, {:.6e}, {:.6}, {:.6e}, {:.6e}",
                layer_name, ec_e, acceptor_dit, donor_dit, f, qit, capture_cross_section_value
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::physics::M_ELECTRON;
    use crate::mesh::mesh_builder::{
        BottomProperties, BulkProperties, FixChargeDensity, InterfaceProperties,
        InterfaceStatesDistribution, PropertyType, SurfaceProperties,
    };
    use tempfile::TempDir;

    fn make_mesh_with_interface(
        potential: Vec<f64>,
        acceptor_dit: Vec<f64>,
        donor_dit: Vec<f64>,
        capture_cross_section: Vec<f64>,
    ) -> MeshStructure {
        MeshStructure {
            id: vec![IDX::Surface, IDX::Interface(0), IDX::Bottom],
            name: vec![
                "Surface".to_string(),
                "Interface".to_string(),
                "Bottom".to_string(),
            ],
            depth: vec![0.0, 1e-9, 2e-9],
            property_type: vec![
                PropertyType::Surface(SurfaceProperties {
                    permittivity: 0.0,
                    delta_conduction_band: 0.0,
                    bandgap_energy: 0.0,
                }),
                PropertyType::Interface(InterfaceProperties {
                    fixcharge_density: FixChargeDensity::Interface(0.0),
                    interface_states: InterfaceStates::Distribution(InterfaceStatesDistribution {
                        id: 0,
                        potential,
                        acceptor_dit,
                        donor_dit,
                        capture_cross_section,
                        mass_electron: 0.2 * M_ELECTRON,
                    }),
                    delta_conduction_band: 0.0,
                }),
                PropertyType::Bottom(BottomProperties {
                    permittivity: 0.0,
                    delta_conduction_band: 0.0,
                    bandgap_energy: 0.0,
                }),
            ],
        }
    }

    fn make_mesh_no_interface() -> MeshStructure {
        MeshStructure {
            id: vec![IDX::Surface, IDX::Bulk(0), IDX::Bottom],
            name: vec![
                "Surface".to_string(),
                "Bulk".to_string(),
                "Bottom".to_string(),
            ],
            depth: vec![0.0, 1e-9, 2e-9],
            property_type: vec![
                PropertyType::Surface(SurfaceProperties {
                    permittivity: 0.0,
                    delta_conduction_band: 0.0,
                    bandgap_energy: 0.0,
                }),
                PropertyType::Bulk(BulkProperties {
                    mass_electron: 0.0,
                    permittivity: 0.0,
                    delta_conduction_band: 0.0,
                    donor_concentration: 0.0,
                    energy_level_donor: 0.0,
                    fixcharge_density: FixChargeDensity::Bulk(0.0),
                    bandgap_energy: 0.0,
                }),
                PropertyType::Bottom(BottomProperties {
                    permittivity: 0.0,
                    delta_conduction_band: 0.0,
                    bandgap_energy: 0.0,
                }),
            ],
        }
    }

    #[test]
    fn test_save_creates_csv_file() {
        let temp_dir = TempDir::new().unwrap();
        let save_dir = temp_dir.path().to_str().unwrap();

        let mesh = make_mesh_with_interface(
            vec![0.5, 1.0, 1.5],
            vec![1e16, 2e16, 3e16],
            vec![4e16, 5e16, 6e16],
            vec![1e-15, 1e-15, 1e-15],
        );
        let occupation: Vec<Option<Vec<f64>>> = vec![None, Some(vec![0.3, 0.5, 0.7]), None];

        save_interface_states(&mesh, &occupation, 1.5, save_dir, 0).unwrap();

        let file_path = temp_dir
            .path()
            .join("interface_states")
            .join("0_0_interfacestates_1.500V.csv");
        assert!(file_path.exists(), "CSV file should be created");
    }

    #[test]
    fn test_save_csv_header_and_row_count() {
        let temp_dir = TempDir::new().unwrap();
        let save_dir = temp_dir.path().to_str().unwrap();

        let mesh = make_mesh_with_interface(vec![1.0], vec![1e16], vec![2e16], vec![1e-15]);
        let occupation: Vec<Option<Vec<f64>>> = vec![None, Some(vec![0.4]), None];

        save_interface_states(&mesh, &occupation, 0.0, save_dir, 1).unwrap();

        let file_path = temp_dir
            .path()
            .join("interface_states")
            .join("1_0_interfacestates_0.000V.csv");
        let content = std::fs::read_to_string(file_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert!(lines[0].contains("Ec-E(eV)"));
        assert!(lines[0].contains("acceptor_like_dit"));
        assert!(lines[0].contains("donor_like_dit"));
        assert!(lines[0].contains("occupation_probability"));
        assert!(lines[0].contains("qit"));
        // 1 header + 1 data row
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_save_no_interface_does_nothing() {
        let temp_dir = TempDir::new().unwrap();
        let save_dir = temp_dir.path().to_str().unwrap();

        let mesh = make_mesh_no_interface();
        let occupation: Vec<Option<Vec<f64>>> = vec![None, None, None];

        save_interface_states(&mesh, &occupation, 0.0, save_dir, 0).unwrap();

        let interface_dir = temp_dir.path().join("interface_states");
        assert!(
            !interface_dir.exists(),
            "No directory should be created when no interface states"
        );
    }

    #[test]
    fn test_save_path_traversal_rejected() {
        let mesh = make_mesh_no_interface();
        let occupation: Vec<Option<Vec<f64>>> = vec![None, None, None];
        let result = save_interface_states(&mesh, &occupation, 0.0, "../evil", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_none_occupation_skips_interface() {
        let temp_dir = TempDir::new().unwrap();
        let save_dir = temp_dir.path().to_str().unwrap();

        let mesh = make_mesh_with_interface(
            vec![0.5, 1.0],
            vec![1e16, 2e16],
            vec![3e16, 4e16],
            vec![1e-15, 1e-15],
        );
        // occupation is None for the interface node
        let occupation: Vec<Option<Vec<f64>>> = vec![None, None, None];

        save_interface_states(&mesh, &occupation, 1.0, save_dir, 0).unwrap();

        let interface_dir = temp_dir.path().join("interface_states");
        assert!(
            !interface_dir.exists(),
            "No directory should be created when occupation is None"
        );
    }
}
