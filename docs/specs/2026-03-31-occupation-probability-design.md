# Design: Interface State Occupation Probability (Qit via Fermi-Dirac / SRH)

**Date**: 2026-03-31
**Branch**: `45-calculate-qit-depends-on-fermi-dirac`

---

## Goal

`InterfaceStatesDistribution` で定義された界面トラップに対して、SRH 統計に基づく `occupation_probability` を計算し、Poisson 方程式の電荷密度（Qit）に反映する。また収束後に `Potential` としてエネルギーグリッド全体の分布を出力する。

---

## Simulation Phases

stress → relief → measurement の3フェーズで occupation probability が変化する。

| フェーズ | 使用時間 | 説明 |
|---------|---------|------|
| Stress | — | ストレス電圧でのフェルミ分布（平衡） |
| Relief | `stress_relief_time` | ストレス後のトラップ放出過程 |
| Measurement | — | Relief 収束後の occupation を固定して CV 測定 |

---

## Data Structure Changes

### `Potential` 構造体に追加（`poisson_solver.rs`）

```rust
pub struct Potential {
    pub depth: Vec<f64>,
    pub potential: Vec<f64>,
    pub electron_density: Vec<f64>,
    pub ionized_donor_concentration: Vec<f64>,
    pub interface_occupation: Vec<Option<Vec<f64>>>,
    // インデックスはメッシュノードと対応。非インターフェースは None。
    // Some(vec) の場合 vec は InterfaceStatesDistribution.potential と同じ長さ。
}
```

### `SimulationPhase` enum を追加（`poisson_solver.rs`）

```rust
pub enum SimulationPhase {
    Stress,
    Relief,       // stress_relief_time を使用
    Measurement,  // previous_phase_occupation を固定値として使用
}
```

### `PoissonSolver` 構造体に追加

```rust
pub struct PoissonSolver {
    // ... 既存フィールド ...
    simulation_phase: SimulationPhase,
    interface_srh: Vec<Option<SRHStatistics>>,
    // インターフェースノードごとに1つ、非インターフェースは None。
    // mass_electron は idx+1 の Bulk ノードから取得して初期化。
    previous_phase_occupation: Vec<Option<Vec<f64>>>,
    // フェーズ切り替え時に前フェーズの収束 occupation を保持し、次フェーズの f₀ として使用。
}
```

---

## Method Changes

### `PoissonSolver::new` の変更

- `simulation_phase = SimulationPhase::Stress` で初期化
- `interface_srh` を初期化:
  - メッシュノードを走査
  - `IDX::Interface` のノードに対して `idx+1` の Bulk ノードの `mass_electron` と `configuration.capture_cross_section.thermal_velocity` から `SRHStatistics::new(temperature, mass_electron, thermal_velocity)` を作成
  - 非インターフェースは `None`
- `previous_phase_occupation`: 全ノード `None` で初期化

### `set_simulation_phase`（新規）

```rust
pub fn set_simulation_phase(&mut self, phase: SimulationPhase)
```

- フェーズを切り替える前に `Potential.interface_occupation` を `previous_phase_occupation` にコピー
- `self.simulation_phase = phase` に更新

### `compute_occupation_probability`（新規、`&self`）

```rust
fn compute_occupation_probability(&self, idx: usize) -> Vec<f64>
```

`InterfaceStatesDistribution` の各エネルギー点 `k` に対して以下を計算:

| フェーズ | 計算式 |
|---------|--------|
| Stress | `f(Et_k) = FD(φ_node - Et_grid[k])` |
| Relief | `f(Et_k) = clamp(f_prev[k] * (1 - eff_emission) + FD(φ_node - Et_grid[k]), max=1.0)` |
| Measurement | `f(Et_k) = f_prev[k]`（固定） |

Relief の補足:
- `eff_emission = interface_srh[idx].effective_emission_coefficient(stress_relief_time, Et_grid[k], σ[k])`
- `FD(φ_node - Et_grid[k])` は `FermiDiracStatistics::fermi_dirac` で計算
- clamp: `f64::min(value, 1.0)`

### `solve_interface` の変更

- `compute_occupation_probability(idx)` を呼び出し Qit を計算して charge balance に加算:

```
Qit = Σ_k [ -Dit_A[k] * f[k] + Dit_D[k] * (1 - f[k]) ] * dE
```

- `dE` は `InterfaceStatesDistribution.potential` の隣接点間距離（または `energy_step`）

### `calculate_interface_occupation`（新規）

```rust
fn calculate_interface_occupation(&mut self)
```

- 全インターフェースノードに `compute_occupation_probability(idx)` を適用
- 結果を `self.potential.interface_occupation[idx]` に格納

### `get_potential_profile` の変更

- 既存の `calculate_electron_density` / `calculate_ionized_donor_concentration` に加えて `calculate_interface_occupation` を呼び出す

### `set_temperature` の変更

- 既存の `donor_activation_model` / `electron_density_model` に加えて `interface_srh` の全要素に `set_temperature` を呼び出す

---

## Physics Summary

- **acceptor-like 状態**: 電子が占有されると負電荷 → 寄与 `-q * Dit_A * f`
- **donor-like 状態**: 電子が放出されると正電荷 → 寄与 `+q * Dit_D * (1 - f)`
- τ(Et) は `SRHStatistics::electron_emission_time(Et_grid[k], σ[k])` で計算（フェルミ準位に依存しない）
