# gan-cv-simulator

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)

`gan-cv-simulator` is a high-performance simulation tool designed to model the Capacitance-Voltage (C-V) characteristics of n-GaN and GaN HEMT diodes. It solves the 1D Poisson equation to determine the potential profile and charge distribution across multi-layer semiconductor structures.

![Simulation Result](./img/rust-gan-cv.png)

## Features

- **Multi-layer Support**: Model complex heterostructures with any number of semiconductor and insulator layers.
- **Physical Accuracy**: Accounts for donor activation, effective mass, and band offsets.
- **Flexible Fixed Charges**: Define bulk fixed charges within layers and interface charges between layers.
- **High Performance**: Implementation in Rust with support for parallel processing (via Rayon) for faster convergence.
- **Interactive Configuration**: Easy-to-use CLI for defining simulation parameters.

## Installation

### From Cargo

```bash
cargo install gan-cv-simulator
```

### Manual Installation

Download the pre-compiled binaries from the [GitHub Releases](https://github.com/WideBandAI/gan-cv-simulator/releases) page.

### From Source

Ensure you have the [Rust toolchain](https://rustup.rs/) installed, then:

```bash
git clone https://github.com/WideBandAI/gan-cv-simulator.git
cd gan-cv-simulator
cargo build --release
```

The binary will be located at `target/release/gan-cv-simulator`.

## Usage

Run the simulator from the terminal:

```bash
gan-cv-simulator
```

The simulator will prompt you interactively for various parameters, including:
- Simulation settings (convergence criteria, relaxation factors)
- Measurement conditions (temperature, voltage sweep)
- Device structure (layer thickness, materials, doping)
- Fixed charge densities
- Mesh discretization

### Documentation

For a detailed explanation of all configuration parameters, please refer to:
- [**Parameter Descriptions**](./docs/parameters.md)

## Outputs

Simulation results are saved in the `outputs/` directory under a subfolder named after your simulation. Key output files include:
- `cv_characteristics.csv`: The calculated capacitance vs. voltage data.
- `potential_profile.csv`: Spatial distribution of the electrostatic potential.
- Plots generated using the `plotters` library.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.