# Simulation Parameters

This document describes the various parameters used in the `gan-cv-simulator`.

## Simulation Settings
These settings control the overall simulation behavior and termination criteria.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Simulation Name` | String | - | Name for this simulation run. Used as the output directory name. |
| `SOR Relaxation Factor` | Float | 1.9 | Successive Over-Relaxation factor for the Poisson solver. |
| `Convergence Criterion` | Float | 1e-6 | Convergence threshold in eV. |
| `Max Iterations` | Integer | 100,000 | Maximum number of iterations for the solver. |
| `Parallel Processing` | Boolean | false | Whether to use parallel processing for the Poisson solver. |

## Measurement Parameters
These parameters define the conditions of the C-V measurement.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Temperature` | Float | 300.0 | Operating temperature in Kelvin (K). |
| `Start Voltage` | Float | - | Initial bias voltage in Volts (V). |
| `End Voltage` | Float | - | Final bias voltage in Volts (V). |
| `Voltage Step` | Float | - | Voltage increment during the sweep in Volts (V). |
| `AC Voltage Amplitude` | Float | 20.0 | Amplitude of the small-signal AC voltage in mV. |
| `Measurement Time` | Float | 100.0 | Total time for the measurement in seconds (s). |
| `Stress Voltage` | Float | 0.0 | Bias voltage applied during the stress phase in Volts (V). |
| `Stress Relief Voltage` | Float | 0.0 | Bias voltage applied during the stress relief phase in Volts (V). |
| `Stress Relief Time` | Float | 0.0 | Duration of the stress relief phase in seconds (s). |

## Device Structure
The device is modeled as a stack of layers. For each layer, the following parameters are required:

| Parameter | Type | Description |
|-----------|------|-------------|
| `Name` | String | Optional name for the layer. |
| `Material Type` | Enum | Semiconductor (s) or Insulator (i). |
| `Thickness` | Float | Thickness of the layer in nanometers (nm). |
| `Relative Permittivity` | Float | Relative dielectric constant ($\epsilon_r$) of the material. |
| `Bandgap Energy` | Float | Bandgap ($E_g$) in electron-Volts (eV). |
| `Delta Conduction Band` | Float | Discontinuity in the conduction band ($\Delta E_c$) from the bottom layer to this layer in eV. |

### Semiconductor-specific Parameters
If a layer is marked as a Semiconductor, these additional parameters are needed:

| Parameter | Type | Description |
|-----------|------|-------------|
| `Effective Mass` | Float | Effective mass coefficient of electrons ($m^* / m_0$). |
| `Donor Concentration` | Float | Concentration of donors in $cm^{-3}$. |
| `Donor Energy Level` | Float | Energy level of donors relative to the conduction band ($E_c - E_d$) in eV. |

## Fixed Charge Parameters
Fixed charges can be defined within bulk layers or at interfaces between layers.

### Bulk Fixed Charge
For each layer defined in the structure:
- `Charge Density`: Fixed charge density in $C/cm^3$. Default is 0.

### Interface Fixed Charge
For each interface between adjacent layers:
- `Charge Density`: Fixed charge density in $C/cm^2$. Default is 0.

## Mesh Parameters
These parameters define the spatial and energy discretization for the simulation.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Number of Mesh Layers` | Integer | 1 | Number of regions with distinct mesh spacing. |
| `Mesh Length` | Float | 0.1 | Spatial grid spacing in nm. |
| `Mesh Layer Thickness` | Float | - | Thickness of each mesh region in nm. |
| `Energy Step` | Float | 0.1 | Discretization step for energy levels in meV. |

## Interface States
Interface trap states can be defined at each interface between adjacent layers. Two models are supported.

### Continuous Interface States (DIGS Model)
The Disorder-Induced Gap States (DIGS) model describes a continuous distribution of traps across the bandgap.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Dit0` | Float | 1e12 | Minimum trap density at the charge neutrality level in $cm^{-2}$. |
| `nssec` | Float | 10 | Ratio $D_{it}(E_c) / D_{it0}$; controls the rise of acceptor-like traps toward $E_c$. |
| `nssev` | Float | 10 | Ratio $D_{it}(E_v) / D_{it0}$; controls the rise of donor-like traps toward $E_v$. |
| `\|Ec - Ecnl\|` | Float | 1.3 | Energy distance from the conduction band to the charge neutrality level in eV. |
| `nd` | Float | 3 | Exponent controlling the energy dependence of donor-like states. |
| `na` | Float | 3 | Exponent controlling the energy dependence of acceptor-like states. |

### Discrete Interface States
Discrete trap levels described by a Gaussian distribution in energy.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Ditmax` | Float | 1e12 | Peak trap density in $cm^{-2}$. |
| `\|Ec - Ed\|` | Float | 0.5 | Energy position of the trap level below the conduction band in eV. |
| `FWHM` | Float | 0.3 | Full width at half maximum of the Gaussian distribution in eV. |
| `State Type` | Enum | - | DonorLike (d) or AcceptorLike (a). |

## Capture Cross-Section
Required for each interface that has interface states. Two models are available.

### Constant Model
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `sigma` | Float | 1e-16 | Capture cross-section in $cm^2$ (energy-independent). |

### Energy-Dependent Model
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `sigma_mid` | Float | 1e-16 | Capture cross-section at the reference energy $E_{mid}$ in $cm^2$. |
| `E_mid` | Float | 0.5 | Reference energy $\|E_c - E_{mid}\|$ in eV. |
| `E_slope` | Float | 0.1 | Energy scale of the exponential variation: $\sigma(E) = \sigma_{mid} \cdot \exp\!\bigl((E - E_{mid}) / E_{slope}\bigr)$ in eV. |

For each interface with states, the effective electron mass used in the thermal velocity calculation is also prompted (defaulting to the mass of the lower layer).

## Boundary Conditions
These parameters define the potential at the device boundaries.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Barrier Height` | Float | - | Schottky barrier height at the surface interface in eV. |
| `Ec - Ef (Bottom)` | Float | Equilibrium | Potential difference between the conduction band and Fermi level at the bottom contact. |
