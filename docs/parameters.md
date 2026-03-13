# Simulation Parameters

This document describes the various parameters used in the `gan-cv-simulator`.

## Simulation Settings
These settings control the overall simulation behavior and termination criteria.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Simulation Name` | String | - | Name for this simulation run. Used as the output directory name. |
| `SOR Relaxation Factor` | Float | 1.9 | Successive Over-Relaxation factor for the Poisson solver. |
| `Convergence Criterion` | Float | 1e-6 | Convergence threshold in eV. |
| `Max Iterations` | Integer | 500,000 | Maximum number of iterations for the solver. |
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

## Boundary Conditions
These parameters define the potential at the device boundaries.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Barrier Height` | Float | - | Schottky barrier height at the surface interface in eV. |
| `Ec - Ef (Bottom)` | Float | Equilibrium | Potential difference between the conduction band and Fermi level at the bottom contact. |
