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

界面（異種材料の接合面）には固定電荷やトラップ準位が存在し、C-V 特性に大きく影響する。このシミュレータでは、界面ノードにおける Poisson 方程式を別途導出し、固定界面電荷と動的なトラップ電荷の両方を考慮する。

### 4.1. 界面ノードの Poisson 方程式

界面ノード $i$ では、電気変位ベクトルの法線成分の不連続条件（Gauss の法則）を離散化する:

$$\frac{\epsilon_{i-1}}{h_u}(\phi_i - \phi_{i-1}) + \frac{\epsilon_{i+1}}{h_d}(\phi_i - \phi_{i+1}) = -q \left( N_{fixed} + N_{it} \right)$$

ここで $c_u = \epsilon_{i-1}/h_u$、$c_d = \epsilon_{i+1}/h_d$ とおくと、SOR 更新式は以下になる:

$$\phi_i^{new} = \frac{c_u \phi_{i-1} + c_d \phi_{i+1} - q(N_{fixed} + N_{it})}{c_u + c_d}$$

- $h_u = x_i - x_{i-1}$: 上側のメッシュ間隔
- $h_d = x_{i+1} - x_i$: 下側のメッシュ間隔
- $\epsilon_{i-1}$, $\epsilon_{i+1}$: 上側・下側層の誘電率 [F/m]
- $N_{fixed}$: 固定界面電荷の面密度 [$m^{-2}$]（正の値 = 正電荷）
- $N_{it}$: 動的トラップ電荷の面密度 [$m^{-2}$]

### 4.2. トラップ電荷面密度

トラップ電荷の面密度 $N_{it}$ は、全トラップエネルギー準位について積算して求める:

$$N_{it} = \sum_k \left[ -D_{it,a}(E_k) \cdot f(E_k) + D_{it,d}(E_k) \cdot \left(1 - f(E_k)\right) \right] \Delta E$$

- $D_{it,a}(E_k)$: アクセプタ型トラップ密度 [$m^{-2} eV^{-1}$]—占有時に負電荷
- $D_{it,d}(E_k)$: ドナー型トラップ密度 [$m^{-2} eV^{-1}$]—空のとき正電荷
- $f(E_k)$: トラップ準位 $E_k$ における占有確率
- $\Delta E$: エネルギーステップ [eV]

### 4.3. トラップ密度モデル

#### 連続モデル（DIGS モデル）

Disorder-Induced Gap States（DIGS）モデルでは、トラップ密度が電荷中立準位 $E_{cnl}$（$= |E_c - E_{cnl}|$ で指定）を基準として指数関数的に増大する:

$$D_{it}(E) = \begin{cases}
D_{it0} \cdot \exp\!\left[\left(\dfrac{|E_{cnl} - E|}{E_{0a}}\right)^{n_a}\right] & E < E_{cnl} \quad \text{(アクセプタ型)}\\[6pt]
D_{it0} \cdot \exp\!\left[\left(\dfrac{|E - E_{cnl}|}{E_{0d}}\right)^{n_d}\right] & E > E_{cnl} \quad \text{(ドナー型)}
\end{cases}$$

エネルギースケールは以下で決まる:

$$E_{0a} = E_{cnl} \cdot \left(\ln N_{ssec}\right)^{-1/n_a}, \qquad E_{0d} = (E_g - E_{cnl}) \cdot \left(\ln N_{ssev}\right)^{-1/n_d}$$

ここで $N_{ssec} = D_{it}(E_c)/D_{it0}$、$N_{ssev} = D_{it}(E_v)/D_{it0}$ はそれぞれ伝導帯・価電子帯端におけるトラップ密度の $D_{it0}$ に対する比率である。

#### 離散モデル（Gaussian 分布）

離散トラップ準位はエネルギー空間上でガウス分布として記述される:

$$D_{it}(E) = D_{it,max} \cdot \exp\!\left(-\frac{(E - E_d)^2}{\sigma_g^2}\right), \qquad \sigma_g^2 = \frac{FWHM^2}{4 \ln 2}$$

- $D_{it,max}$: ピークトラップ密度 [$m^{-2}$]
- $E_d = |E_c - E_d|$: 伝導帯から測ったトラップ準位の深さ [eV]
- $FWHM$: 分布の半値全幅 [eV]

### 4.4. 占有確率

トラップの占有確率は、Fermi-Dirac 平衡占有とSRH電子放出による非平衡フロアの最大値として定義される:

$$f(E_t) = \max\!\left(f_{eq}(E_t),\; f_{prev}(E_t) \cdot \left(1 - \xi_{em}\right)\right)$$

**Fermi-Dirac 平衡占有確率**:

$$f_{eq}(E_t) = \frac{1}{1 + \exp\!\left(\dfrac{E_t - E_f}{k_B T / q}\right)}$$

ここで $E_t - E_f = \phi_{node} - (E_c - E_t)$。$\phi_{node} = E_c - E_f$ は界面ノードにおける伝導帯とフェルミ準位の差（eV）である。

**有効放出係数**:

$$\xi_{em} = 1 - \exp\!\left(-\frac{t}{\tau_{em}}\right)$$

**電子放出時定数**:

$$\tau_{em}(E_t) = \frac{\exp\!\left(\dfrac{E_c - E_t}{k_B T / q}\right)}{v_{th} \cdot \sigma(E_t) \cdot N_c}$$

- $t$: 測定時刻（電圧ステップ積算時間） [s]
- $v_{th} = \sqrt{3 k_B T / m^*}$: 熱速度 [m/s]
- $\sigma(E_t)$: トラップ準位 $E_t$ における捕獲断面積 [$m^2$]
- $N_c$: 伝導帯有効状態密度 [$m^{-3}$]

$f_{prev}$ は直前の電圧ステップにおける占有確率であり、バイアスを逆方向に変化させた際のヒステリシスをモデル化する。

### 4.5. 捕獲断面積モデル

#### 定数モデル

$$\sigma(E_t) = \sigma_0$$

#### エネルギー依存モデル

$$\sigma(E_t) = \sigma_{mid} \cdot \exp\!\left(\frac{E_t - E_{mid}}{E_{slope}}\right)$$

- $\sigma_{mid}$: 基準エネルギー $E_{mid}$ における捕獲断面積 [$m^2$]
- $E_{mid} = |E_c - E_{mid}|$: 基準エネルギーの伝導帯からの深さ [eV]
- $E_{slope}$: 指数変化のエネルギースケール [eV]（正の値 = $E_{mid}$ より深いトラップで $\sigma$ が増大）

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