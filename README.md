# gan-cv-simulator

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)

`gan-cv-simulator` is a high-performance simulation tool designed to model the Capacitance-Voltage (C-V) characteristics of n-GaN and GaN HEMT diodes. It solves the 1D Poisson equation to determine the potential profile and charge distribution across multi-layer semiconductor structures.

![gan-cv-simulator_image](./img/rust-gan-cv.png)

## Physics & Modeling: DIGS Integration
The most distinctive feature of this simulator is its specialized support for the **Disorder-Induced Gap State (DIGS) model**, enabling precise analysis of interface physics.

* **Interface State Density:** Define custom distributions for interface state densities to see their impact on C-V curves.
* **High-Frequency Focus:** Optimized for the regime where **electron capture** is dominant (typically **1 MHz or higher**).
* **Note on Limitations:** It does **not** account for the full emission cycles seen in low-frequency measurements. For the best accuracy, compare results against **high-frequency experimental data**.



## Core Capabilities

-   **Multi-layer Heterostructures:** Model complex stacks with any number of semiconductor and insulator layers.
-   **Physical Precision:** Accounts for donor activation, effective mass, and band offsets.
-   **Flexible Charge Distribution:** Define bulk fixed charges within layers and interface charges between them.
-   **High Performance:** Built with **Rust and Rayon** for efficient parallel processing and faster convergence.
-   **Interactive CLI:** A user-friendly command-line interface guides you through the parameter setup.

---

## Installation

### From Cargo
```bash
cargo install gan-cv-simulator
````

### Manual Installation

Download pre-compiled binaries from the [GitHub Releases](https://github.com/WideBandAI/gan-cv-simulator/releases) page.

### From Source

```bash
git clone [https://github.com/WideBandAI/gan-cv-simulator.git](https://github.com/WideBandAI/gan-cv-simulator.git)
cd gan-cv-simulator
cargo build --release
```

## Usage

Run the simulator from the terminal:

```bash
gan-cv-simulator
```

The interactive CLI will prompt you for:

  * **Simulation & Mesh:** Convergence criteria, relaxation factors, and discretization.
  * **Device Structure:** Layer thickness, materials, doping, and fixed charges.
  * **Interface Physics:** Continuous DIGS model, discrete Gaussian traps, and energy-dependent capture cross-sections.
  * **Measurement Conditions:** Temperature, voltage sweep, and stress conditions.

### Documentation

  * [**Parameter Descriptions**](https://www.google.com/search?q=./docs/parameters.md): Detailed explanation of all inputs.
  * [**Simulation Physics**](https://www.google.com/search?q=./docs/physics.md): Mathematical models and equations.

## Outputs

Results are saved in the `outputs/` directory:

  - `cv_characteristics.csv`: Capacitance vs. Voltage data.
  - `potential_profile.csv`: Spatial distribution of electrostatic potential.
  - Visual plots generated using the `plotters` library.

## Citation

If you use this software in your research, please cite the following paper:

```bibtex
@article{nishiguchi2017current,
  title={Current linearity and operation stability in Al2O3-gate AlGaN/GaN MOS high electron mobility transistors},
  author={Nishiguchi, Kenya and Kaneki, Syota and Ozaki, Shiro and Hashizume, Tamotsu},
  journal={Japanese Journal of Applied Physics},
  volume={56},
  number={10},
  pages={101001},
  year={2017},
  publisher={The Japan Society of Applied Physics}
}
```

## License

Licensed under the Apache License 2.0. See [LICENSE](https://www.google.com/search?q=LICENSE) for details.
