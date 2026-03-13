# Simulation Physics and Formulas

This document explains the physical equations and mathematical models used in the `gan-cv-simulator`.

## 1. Poisson's Equation

The core of the simulator is the one-dimensional (1D) Poisson equation, which relates the electrostatic potential $\phi(x)$ to the space charge density $\rho(x)$:

$$\frac{d}{dx} \left( \epsilon(x) \frac{d\phi(x)}{dx} \right) = -\rho(x)$$

Where:
- $\phi(x)$: Electrostatic potential [V]
- $\epsilon(x)$: Static permittivity [F/m]
- $\rho(x)$: Net space charge density [$C/m^3$]

In this simulator, we solve for $\phi(x) = E_c(x) - E_f$, where $E_c$ is the conduction band edge and $E_f$ is the Fermi level.

### Discretization
The equation is discretized using a finite difference method. For a bulk node $i$ with mesh spacing $h_{u} = x_i - x_{i-1}$ and $h_{d} = x_{i+1} - x_i$:

$$\phi_i^{new} = \phi_i^{old} + \omega \cdot \Delta \phi_i$$

The Successive Over-Relaxation (SOR) method is used to iterate towards convergence, where $\omega$ is the relaxation factor.

## 2. Charge Density Models

The net charge density $\rho(x)$ is given by:

$$\rho(x) = q \cdot (N_D^+(x) - n(x) + \rho_{fixed}(x))$$

Where:
- $q$: Elementary charge [$1.602 \times 10^{-19}$ C]
- $N_D^+(x)$: Ionized donor concentration [$m^{-3}$]
- $n(x)$: Free electron concentration [$m^{-3}$]
- $\rho_{fixed}(x)$: Fixed charge density [$C/m^3$]

### 2.1. Consideration of Carriers
In wide-bandgap semiconductors like GaN, minority carrier generation is extremely slow at room temperature. Under negative bias, GaN MOS/MIS structures typically exhibit **deep depletion** rather than inversion because the minority carrier (hole) density is too small to be observed in standard C-V measurements within a reasonable timeframe.

Furthermore, this project specifically targets **n-GaN** and GaN HEMT structures; **p-GaN is currently out of scope**. Therefore, the Poisson equation implemented here only considers ionized donors ($N_D^+$) and electrons ($n$), neglecting hole density ($p$) and ionized acceptors ($N_A^-$).

### 2.2. Electron Density (Boltzmann Approximation)
Assuming the degenerate case is not dominant, the free electron density $n$ is calculated using Boltzmann statistics:

$$n = N_c \exp\left( -\frac{q \phi}{k_B T} \right)$$

Where $N_c$ is the effective density of states in the conduction band:

$$N_c = 2 \left( \frac{2\pi m^* k_B T}{h^2} \right)^{3/2}$$

- $m^*$: Effective mass of electron
- $k_B$: Boltzmann constant
- $T$: Temperature [K]
- $h$: Planck constant

### 2.3. Donor Ionization
The ionized donor concentration $N_D^+$ is modeled using the impurity distribution:

$$N_D^+ = \frac{N_D}{1 + g \exp\left( \frac{E_f - E_d}{k_B T} \right)} = \frac{N_D}{1 + 2 \exp\left( -\frac{q(\phi - \Delta E_d)}{k_B T} \right)}$$

Where:
- $N_D$: Total donor concentration
- $g$: Degeneracy factor (typically 2 for donors)
- $\Delta E_d$: Donor ionization energy ($E_c - E_d$)

## 3. Boundary Conditions

- **Surface (Gate)**: Dirichlet boundary condition based on the Schottky barrier height $\phi_B$ and applied gate voltage $V_g$:
  $$\phi(0) = \phi_B - V_g$$
- **Bottom (Substrate)**: Dirichlet boundary condition based on the equilibrium potential of the bulk semiconductor.

## 4. Capacitance Calculation

The differential capacitance $C$ is calculated using the small-signal AC method:

$$C(V_g) = \frac{dQ_{total}}{dV_g} \approx \frac{Q_{total}(V_g + \Delta V) - Q_{total}(V_g - \Delta V)}{2 \Delta V}$$

Where $Q_{total}$ is the total integrated electron charge in the device:

$$Q_{total} = q \int n(x) dx$$

The simulator uses a small AC amplitude $\Delta V$ (typically 20 mV) to numerically evaluate this derivative.

## References

- @article{nishiguchi2022numerical,
  title={A numerical modeling of the frequency dependence of the capacitance--voltage and conductance--voltage characteristics of GaN MIS structures},
  author={Nishiguchi, K and Nakata, K and Hashizume, T},
  journal={Journal of Applied Physics},
  volume={132},
  number={17},
  year={2022},
  publisher={AIP Publishing}
}