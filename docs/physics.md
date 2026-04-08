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

$$\rho(x) = q \cdot (N_D^+(x) - n(x) + N_{fixed}(x))$$

Where:
- $q$: Elementary charge [$1.602 \times 10^{-19}$ C]
- $N_D^+(x)$: Ionized donor concentration [$m^{-3}$]
- $n(x)$: Free electron concentration [$m^{-3}$]
- $N_{fixed}(x)$: Fixed charge number density [$m^{-3}$]

### 2.1. Consideration of Carriers
In wide-bandgap semiconductors like GaN, minority carrier generation is extremely slow at room temperature. [Under negative bias, GaN MOS/MIS structures typically exhibit **deep depletion** rather than inversion because the minority carrier (hole) density is too small to be observed in standard C-V measurements within a reasonable timeframe.](#reference-1) 

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

## 4. Interface States

Fixed charges and trap states exist at heterojunction interfaces and significantly affect C-V characteristics. This simulator treats interface nodes with a separate formulation of Poisson's equation that accounts for both fixed interface charges and dynamic trap charges.

### 4.1. Poisson's Equation at an Interface Node

At interface node $i$, the discontinuity condition for the normal component of the electric displacement field (Gauss's law) is discretized as:

$$\frac{\epsilon_{i-1}}{h_u}(\phi_i - \phi_{i-1}) + \frac{\epsilon_{i+1}}{h_d}(\phi_i - \phi_{i+1}) = -q \left( N_{fixed} + N_{it} \right)$$

Defining $c_u = \epsilon_{i-1}/h_u$ and $c_d = \epsilon_{i+1}/h_d$, the SOR update equation becomes:

$$\phi_i^{new} = \frac{c_u \phi_{i-1} + c_d \phi_{i+1} - q(N_{fixed} + N_{it})}{c_u + c_d}$$

Where:
- $h_u = x_i - x_{i-1}$: mesh spacing above the interface
- $h_d = x_{i+1} - x_i$: mesh spacing below the interface
- $\epsilon_{i-1}$, $\epsilon_{i+1}$: permittivity of the upper and lower layers [F/m]
- $N_{fixed}$: fixed interface charge areal density [$m^{-2}$] (positive = positive charge)
- $N_{it}$: dynamic trap charge areal density [$m^{-2}$]

### 4.2. Trap Charge Areal Density

The net trap charge areal density $N_{it}$ is obtained by integrating over all trap energy levels:

$$N_{it} = \sum_k \left[ -D_{it,a}(E_k) \cdot f(E_k) + D_{it,d}(E_k) \cdot \left(1 - f(E_k)\right) \right] \Delta E$$

Where:
- $D_{it,a}(E_k)$: acceptor-like trap density at energy $E_k$ [$m^{-2} eV^{-1}$] — negatively charged when occupied
- $D_{it,d}(E_k)$: donor-like trap density at energy $E_k$ [$m^{-2} eV^{-1}$] — positively charged when empty
- $f(E_k)$: occupation probability of the trap state at energy $E_k$
- $\Delta E$: energy step [eV]

### 4.3. Trap Density Models

#### Continuous Model (DIGS Model)

In the Disorder-Induced Gap States (DIGS) model, the trap density increases exponentially from the charge neutrality level $E_{cnl}$ (specified as $|E_c - E_{cnl}|$):

$$D_{it}(E) = \begin{cases}
D_{it0} \cdot \exp\!\left[\left(\dfrac{|E_{cnl} - E|}{E_{0a}}\right)^{n_a}\right] & E < E_{cnl} \quad \text{(acceptor-like)}\\[6pt]
D_{it0} \cdot \exp\!\left[\left(\dfrac{|E - E_{cnl}|}{E_{0d}}\right)^{n_d}\right] & E > E_{cnl} \quad \text{(donor-like)}
\end{cases}$$

The energy scale parameters are determined by:

$$E_{0a} = E_{cnl} \cdot \left(\ln N_{ssec}\right)^{-1/n_a}, \qquad E_{0d} = (E_g - E_{cnl}) \cdot \left(\ln N_{ssev}\right)^{-1/n_d}$$

Where $N_{ssec} = D_{it}(E_c)/D_{it0}$ and $N_{ssev} = D_{it}(E_v)/D_{it0}$ are the ratios of the trap density at the conduction and valence band edges relative to $D_{it0}$, respectively.

#### Discrete Model (Gaussian Distribution)

Discrete trap levels are described by a Gaussian distribution in energy:

$$D_{it}(E) = D_{it,max} \cdot \exp\!\left(-\frac{(E - E_d)^2}{\sigma_g^2}\right), \qquad \sigma_g^2 = \frac{FWHM^2}{4 \ln 2}$$

Where:
- $D_{it,max}$: peak trap density [$m^{-2}$]
- $E_d = |E_c - E_d|$: trap level depth below the conduction band [eV]
- $FWHM$: full width at half maximum of the distribution [eV]

### 4.4. Occupation Probability

The trap occupation probability is defined as the maximum of the Fermi-Dirac equilibrium occupation and a non-equilibrium floor set by SRH electron emission:

$$f(E_t) = \max\!\left(f_{eq}(E_t),\; f_{prev}(E_t) \cdot \left(1 - \xi_{em}\right)\right)$$

**Fermi-Dirac equilibrium occupation**:

$$f_{eq}(E_t) = \frac{1}{1 + \exp\!\left(\dfrac{E_t - E_f}{k_B T / q}\right)}$$

Where $E_t - E_f = \phi_{node} - (E_c - E_t)$, and $\phi_{node} = E_c - E_f$ is the potential at the interface node in eV.

**Effective emission coefficient**:

$$\xi_{em} = 1 - \exp\!\left(-\frac{t}{\tau_{em}}\right)$$

**Electron emission time constant**:

$$\tau_{em}(E_t) = \frac{\exp\!\left(\dfrac{E_c - E_t}{k_B T / q}\right)}{v_{th} \cdot \sigma(E_t) \cdot N_c}$$

Where:
- $t$: measurement time (cumulative time from voltage sweep start) [s]
- $v_{th} = \sqrt{3 k_B T / m^*}$: thermal velocity [m/s]
- $\sigma(E_t)$: capture cross-section at trap energy $E_t$ [$m^2$]
- $N_c$: effective density of states in the conduction band [$m^{-3}$]

$f_{prev}$ is the occupation probability from the previous voltage step, modeling the hysteresis observed when the bias is swept in the reverse direction.

### 4.5. Capture Cross-Section Models

#### Constant Model

$$\sigma(E_t) = \sigma_0$$

#### Energy-Dependent Model

$$\sigma(E_t) = \sigma_{mid} \cdot \exp\!\left(\frac{E_t - E_{mid}}{E_{slope}}\right)$$

Where:
- $\sigma_{mid}$: capture cross-section at the reference energy $E_{mid}$ [$m^2$]
- $E_{mid} = |E_c - E_{mid}|$: depth of the reference energy below the conduction band [eV]
- $E_{slope}$: energy scale of the exponential variation [eV] (positive value = $\sigma$ increases for traps deeper than $E_{mid}$)

## 5. Capacitance Calculation

The differential capacitance $C$ is calculated using the small-signal AC method:

$$C(V_g) = \frac{dQ_{total}}{dV_g} \approx \frac{Q_{total}(V_g + \Delta V) - Q_{total}(V_g - \Delta V)}{2 \Delta V}$$

Where $Q_{total}$ is the total integrated electron charge in the device:

$$Q_{total} = q \int n(x) dx$$

The simulator uses a small AC amplitude $\Delta V$ (typically 20 mV) to numerically evaluate this derivative.

## References
### Reference 1
```bibtex
@article{nishiguchi2022numerical,
  title={A numerical modeling of the frequency dependence of the capacitance--voltage and conductance--voltage characteristics of GaN MIS structures},
  author={Nishiguchi, K and Nakata, K and Hashizume, T},
  journal={Journal of Applied Physics},
  volume={132},
  number={17},
  year={2022},
  publisher={AIP Publishing}
}
```