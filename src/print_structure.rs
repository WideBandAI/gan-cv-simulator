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
        let name = device.name.get(i).map(|s| s.as_str()).unwrap_or("Unknown");
        let mat = match device.material_type.get(i) {
            Some(MaterialType::Semiconductor) => "Semiconductor",
            Some(MaterialType::Insulator) => "Insulator",
            None => "Unknown",
        };
        println!(
            "| {:<w0$} | {:<w1$} | {:>w2$.2} | {:>w3$.3} | {:>w4$.3} | {:>w5$.3e} |",
            name,
            mat,
            device.thickness.get(i).unwrap_or(&0.0) * M_TO_NM,
            device.bandgap_energy.get(i).unwrap_or(&0.0),
            device.delta_conduction_band.get(i).unwrap_or(&0.0),
            device.donor_concentration.get(i).unwrap_or(&0.0) * PER_M3_TO_PER_CM3,
            w0 = col_widths[0],
            w1 = col_widths[1],
            w2 = col_widths[2],
            w3 = col_widths[3],
            w4 = col_widths[4],
            w5 = col_widths[5],
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
        println!(
            "|{}|",
            build_title_row("Continuous Interface States (DIGS)", &col_widths)
        );
        println!("{}", sep);
        println!("|{}|", build_header_row(&headers, &col_widths));
        println!("{}", sep);
        for (idx, &iface_id) in continuous.interface_id.iter().enumerate() {
            let i = iface_id as usize;
            let iface_name = format!(
                "{} / {}",
                device.name.get(i).map(|s| s.as_str()).unwrap_or("?"),
                device.name.get(i + 1).map(|s| s.as_str()).unwrap_or("?")
            );
            let p = &continuous.parameters[idx];
            println!(
                "| {:<w0$} | {:>w1$.3e} | {:>w2$.2} | {:>w3$.2} | {:>w4$.3} | {:>w5$.2} | {:>w6$.2} |",
                iface_name,
                p.dit0 * PER_M2_TO_PER_CM2,
                p.nssec,
                p.nssev,
                p.ecnl,
                p.nd,
                p.na,
                w0 = col_widths[0],
                w1 = col_widths[1],
                w2 = col_widths[2],
                w3 = col_widths[3],
                w4 = col_widths[4],
                w5 = col_widths[5],
                w6 = col_widths[6],
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
        println!(
            "|{}|",
            build_title_row("Discrete Interface States", &col_widths)
        );
        println!("{}", sep);
        println!("|{}|", build_header_row(&headers, &col_widths));
        println!("{}", sep);
        for (idx, &iface_id) in discrete.interface_id.iter().enumerate() {
            let i = iface_id as usize;
            let iface_name = format!(
                "{} / {}",
                device.name.get(i).map(|s| s.as_str()).unwrap_or("?"),
                device.name.get(i + 1).map(|s| s.as_str()).unwrap_or("?")
            );
            for (trap_idx, model) in discrete.parameters[idx].iter().enumerate() {
                println!(
                    "| {:<w0$} | {:>w1$} | {:>w2$.3e} | {:>w3$.3} | {:>w4$.3} | {:<w5$} |",
                    iface_name,
                    trap_idx,
                    model.ditmax() * PER_M2_TO_PER_CM2,
                    model.ed(),
                    model.fwhm(),
                    model.state_type().to_string(),
                    w0 = col_widths[0],
                    w1 = col_widths[1],
                    w2 = col_widths[2],
                    w3 = col_widths[3],
                    w4 = col_widths[4],
                    w5 = col_widths[5],
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

fn build_title_row(title: &str, col_widths: &[usize]) -> String {
    let total = col_widths.iter().map(|&w| w + 2).sum::<usize>() + col_widths.len() - 1;
    format!(" {:<total$} ", title, total = total)
}
