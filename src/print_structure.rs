use crate::config::interface_states::{
    ContinuousInterfaceStatesConfig, DiscreteInterfaceStatesConfig,
};
use crate::config::structure::{DeviceStructure, MaterialType};
use crate::constants::units::{M_TO_NM, PER_M2_TO_PER_CM2, PER_M3_TO_PER_CM3};

/// Prints a table of layer properties in nvidia-smi style.
pub fn print_layer_structure(device: &DeviceStructure) {
    let col_widths: [usize; 6] = [15, 13, 10, 8, 8, 14];
    let headers = [
        "Layer",
        "Material",
        "Thick(nm)",
        "Eg(eV)",
        "dEc(eV)",
        "Nd(cm^-3)",
    ];

    let sep = build_sep(&col_widths);
    println!("\n{}", sep);
    println!("|{}|", build_header_row(&headers, &col_widths));
    println!("{}", sep);

    for i in 0..device.id.len() {
        let mat = match device.material_type[i] {
            MaterialType::Semiconductor => "Semiconductor",
            MaterialType::Insulator => "Insulator",
        };
        println!(
            "| {:<15} | {:<13} | {:>10.2} | {:>8.3} | {:>8.3} | {:>14.3e} |",
            device.name[i],
            mat,
            device.thickness[i] * M_TO_NM,
            device.bandgap_energy[i],
            device.delta_conduction_band[i],
            device.donor_concentration[i] * PER_M3_TO_PER_CM3,
        );
    }

    println!("{}\n", sep);
}

/// Prints tables of continuous and discrete interface states when any are configured.
pub fn print_interface_states(
    device: &DeviceStructure,
    continuous: &ContinuousInterfaceStatesConfig,
    discrete: &DiscreteInterfaceStatesConfig,
) {
    if continuous.interface_id.is_empty() && discrete.interface_id.is_empty() {
        return;
    }

    if !continuous.interface_id.is_empty() {
        let col_widths: [usize; 7] = [23, 14, 6, 6, 11, 6, 6];
        let headers = [
            "Interface",
            "Dit0(cm^-2)",
            "nssec",
            "nssev",
            "Ec-Ecnl(eV)",
            "nd",
            "na",
        ];
        let sep = build_sep(&col_widths);
        println!("\n{}", sep);
        println!("|{}|", build_header_row(&headers, &col_widths));
        println!("{}", sep);
        for (idx, &iface_id) in continuous.interface_id.iter().enumerate() {
            let i = iface_id as usize;
            let iface_name = format!("{} / {}", device.name[i], device.name[i + 1]);
            let p = &continuous.parameters[idx];
            println!(
                "| {:<23} | {:>14.3e} | {:>6.2} | {:>6.2} | {:>11.3} | {:>6.2} | {:>6.2} |",
                iface_name,
                p.dit0 * PER_M2_TO_PER_CM2,
                p.nssec,
                p.nssev,
                p.ecnl,
                p.nd,
                p.na,
            );
        }
        println!("{}\n", sep);
    }

    if !discrete.interface_id.is_empty() {
        let col_widths: [usize; 6] = [23, 5, 14, 10, 8, 13];
        let headers = [
            "Interface",
            "Trap#",
            "Ditmax(cm^-2)",
            "|Ec-Ed|(eV)",
            "FWHM(eV)",
            "Type",
        ];
        let sep = build_sep(&col_widths);
        println!("\n{}", sep);
        println!("|{}|", build_header_row(&headers, &col_widths));
        println!("{}", sep);
        for (idx, &iface_id) in discrete.interface_id.iter().enumerate() {
            let i = iface_id as usize;
            let iface_name = format!("{} / {}", device.name[i], device.name[i + 1]);
            for (trap_idx, model) in discrete.parameters[idx].iter().enumerate() {
                println!(
                    "| {:<23} | {:>5} | {:>14.3e} | {:>10.3} | {:>8.3} | {:<13} |",
                    iface_name,
                    trap_idx,
                    model.ditmax() * PER_M2_TO_PER_CM2,
                    model.ed(),
                    model.fwhm(),
                    model.state_type().to_string(),
                );
            }
        }
        println!("{}\n", sep);
    }
}

fn build_sep(col_widths: &[usize]) -> String {
    let inner = col_widths
        .iter()
        .map(|&w| "-".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("+");
    format!("+{}+", inner)
}

fn build_header_row(headers: &[&str], col_widths: &[usize]) -> String {
    headers
        .iter()
        .zip(col_widths.iter())
        .map(|(h, &w)| format!(" {:^w$} ", h, w = w))
        .collect::<Vec<_>>()
        .join("|")
}
