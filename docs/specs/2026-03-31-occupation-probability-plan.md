# Occupation Probability (Qit via SRH) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `InterfaceStatesDistribution` のトラップに対して SRH 統計による `occupation_probability` を計算し、Poisson 方程式の Qit 電荷密度に反映しつつ、収束後の全エネルギーグリッド分布を `Potential` として出力する。

**Architecture:** `PoissonSolver` に `SimulationPhase`・`SRHStatistics`・`FermiDiracStatistics` を保持し、SOR ループ中は `compute_occupation_probability` をオンザフライ呼び出し。収束後に `calculate_interface_occupation` で `Potential.interface_occupation` を埋める。

**Tech Stack:** Rust, `physics_equations::srh_statistics::SRHStatistics`, `physics_equations::fermi_dirac::FermiDiracStatistics`

---

## File Map

| ファイル | 変更種別 | 内容 |
|---------|---------|------|
| `src/solvers/poisson_solver.rs` | Modify | 全変更の中心。enum・フィールド追加・新メソッド・既存メソッド変更 |
| `src/solvers/cv_solver.rs` | Modify | テスト内の `PoissonSolver::new` 呼び出しに引数追加 |
| `src/main.rs` | Modify | `PoissonSolver::new` 呼び出しに実際の config 値を渡す |

---

## Task 1: `SimulationPhase` enum と `Potential.interface_occupation` フィールドの追加

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

- [ ] **Step 1: 失敗するテストを書く**

`src/solvers/poisson_solver.rs` のテストモジュール末尾（`}` の直前）に追加:

```rust
// -----------------------------------------------------------------------
// interface_occupation フィールド初期化テスト
// -----------------------------------------------------------------------

/// Potential の interface_occupation がメッシュ長と同じサイズで初期化されること
#[test]
fn test_potential_interface_occupation_initialized_with_none() {
    let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
    let n = mesh.id.len();
    let solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false);
    assert_eq!(solver.potential.interface_occupation.len(), n);
    for occ in &solver.potential.interface_occupation {
        assert!(occ.is_none());
    }
}
```

- [ ] **Step 2: テストを実行して失敗を確認**

```bash
cargo test test_potential_interface_occupation_initialized_with_none 2>&1 | tail -20
```

Expected: コンパイルエラーまたは `field does not exist`

- [ ] **Step 3: `SimulationPhase` enum を追加**

`src/solvers/poisson_solver.rs` の `pub struct Potential {` より前（`use` 文の後）に追加:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationPhase {
    Stress,
    Relief,
    Measurement,
}
```

- [ ] **Step 4: `Potential` 構造体に `interface_occupation` フィールドを追加**

```rust
// 変更前
pub struct Potential {
    pub depth: Vec<f64>,
    pub potential: Vec<f64>,
    pub electron_density: Vec<f64>,
    pub ionized_donor_concentration: Vec<f64>,
}
```

```rust
// 変更後
pub struct Potential {
    pub depth: Vec<f64>,
    pub potential: Vec<f64>,
    pub electron_density: Vec<f64>,
    pub ionized_donor_concentration: Vec<f64>,
    pub interface_occupation: Vec<Option<Vec<f64>>>,
}
```

- [ ] **Step 5: `PoissonSolver::new` 内の `Potential` 初期化に新フィールドを追加**

```rust
// 変更前
let potential = Potential {
    depth: mesh_structure.depth.clone(),
    potential: vec![initial_potential; mesh_structure.id.len()],
    electron_density: vec![0.0; mesh_structure.id.len()],
    ionized_donor_concentration: vec![0.0; mesh_structure.id.len()],
};
```

```rust
// 変更後
let n = mesh_structure.id.len();
let potential = Potential {
    depth: mesh_structure.depth.clone(),
    potential: vec![initial_potential; n],
    electron_density: vec![0.0; n],
    ionized_donor_concentration: vec![0.0; n],
    interface_occupation: vec![None; n],
};
```

- [ ] **Step 6: テストを実行してパスを確認**

```bash
cargo test test_potential_interface_occupation_initialized_with_none 2>&1 | tail -10
```

Expected: `test ... ok`

- [ ] **Step 7: 全テストが通ることを確認**

```bash
cargo test 2>&1 | tail -20
```

Expected: 全テスト ok

- [ ] **Step 8: コミット**

```bash
git add src/solvers/poisson_solver.rs
git commit -m "feat: add SimulationPhase enum and Potential.interface_occupation field"
```

---

## Task 2: `PoissonSolver` に新フィールド追加・`new()` シグネチャ変更・全呼び出し箇所更新

**Files:**
- Modify: `src/solvers/poisson_solver.rs`
- Modify: `src/solvers/cv_solver.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 失敗するテストを書く**

`src/solvers/poisson_solver.rs` のテストモジュールに追加:

```rust
/// interface ノードに対して SRHStatistics が Some で初期化されること
#[test]
fn test_new_interface_srh_some_for_interface_nodes() {
    let mesh = make_interface_mesh(10.0 * EPSILON_0, 0.0);
    // make_interface_mesh: [Surface, Bulk(0), Interface(0), Bulk(1), Bottom]
    // Interface は idx=2
    let solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false,
        2.6e5, // thermal_velocity
        0.0,   // stress_relief_time
    );
    assert!(solver.interface_srh[0].is_none(), "Surface should be None");
    assert!(solver.interface_srh[1].is_none(), "Bulk should be None");
    assert!(solver.interface_srh[2].is_some(), "Interface(0) should be Some");
    assert!(solver.interface_srh[3].is_none(), "Bulk should be None");
    assert!(solver.interface_srh[4].is_none(), "Bottom should be None");
}

/// simulation_phase が Stress で初期化されること
#[test]
fn test_new_simulation_phase_is_stress() {
    let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
    let solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 0.0,
    );
    assert_eq!(solver.simulation_phase, SimulationPhase::Stress);
}
```

- [ ] **Step 2: テストを実行して失敗を確認**

```bash
cargo test test_new_interface_srh_some_for_interface_nodes 2>&1 | tail -10
```

Expected: コンパイルエラー（フィールド・引数が存在しない）

- [ ] **Step 3: `use` 文に import を追加**

`src/solvers/poisson_solver.rs` の import セクションに追加:

```rust
use crate::mesh_builder::mesh_builder::InterfaceStates;
use crate::physics_equations::fermi_dirac::FermiDiracStatistics;
use crate::physics_equations::srh_statistics::SRHStatistics;
```

- [ ] **Step 4: `PoissonSolver` 構造体に新フィールドを追加**

```rust
// 変更前
pub struct PoissonSolver {
    pub potential: Potential,
    pub mesh_structure: MeshStructure,
    pub temperature: f64,
    sor_relaxation_factor: f64,
    red_indices: Vec<usize>,
    black_indices: Vec<usize>,
    convergence_threshold: f64,
    max_iterations: usize,
    electron_density_model: Box<dyn ElectronDensity>,
    donor_activation_model: DonorActivation,
    parallel_use: bool,
}
```

```rust
// 変更後
pub struct PoissonSolver {
    pub potential: Potential,
    pub mesh_structure: MeshStructure,
    pub temperature: f64,
    sor_relaxation_factor: f64,
    red_indices: Vec<usize>,
    black_indices: Vec<usize>,
    convergence_threshold: f64,
    max_iterations: usize,
    electron_density_model: Box<dyn ElectronDensity>,
    donor_activation_model: DonorActivation,
    parallel_use: bool,
    pub simulation_phase: SimulationPhase,
    pub interface_srh: Vec<Option<SRHStatistics>>,
    pub previous_phase_occupation: Vec<Option<Vec<f64>>>,
    fermi_dirac: FermiDiracStatistics,
    stress_relief_time: f64,
}
```

- [ ] **Step 5: `PoissonSolver::new` のシグネチャと本体を更新**

```rust
// 変更前
pub fn new(
    mesh_structure: MeshStructure,
    initial_potential: f64,
    temperature: f64,
    sor_relaxation_factor: f64,
    convergence_threshold: f64,
    max_iterations: usize,
    parallel_use: bool,
) -> Self {
    let n = mesh_structure.id.len();
    let potential = Potential { ... };
    let red_indices = ...;
    let black_indices = ...;
    Self {
        potential,
        mesh_structure,
        temperature,
        sor_relaxation_factor,
        red_indices,
        black_indices,
        convergence_threshold,
        max_iterations,
        electron_density_model: Box::new(BoltzmannApproximation::new(temperature)),
        donor_activation_model: DonorActivation::new(temperature),
        parallel_use,
    }
}
```

```rust
// 変更後
pub fn new(
    mesh_structure: MeshStructure,
    initial_potential: f64,
    temperature: f64,
    sor_relaxation_factor: f64,
    convergence_threshold: f64,
    max_iterations: usize,
    parallel_use: bool,
    thermal_velocity: f64,
    stress_relief_time: f64,
) -> Self {
    let n = mesh_structure.id.len();
    let potential = Potential {
        depth: mesh_structure.depth.clone(),
        potential: vec![initial_potential; n],
        electron_density: vec![0.0; n],
        ionized_donor_concentration: vec![0.0; n],
        interface_occupation: vec![None; n],
    };
    let red_indices: Vec<usize> = (1..n - 1).filter(|i| i % 2 == 1).collect();
    let black_indices: Vec<usize> = (1..n - 1).filter(|i| i % 2 == 0).collect();
    let interface_srh: Vec<Option<SRHStatistics>> = (0..n)
        .map(|idx| {
            if matches!(mesh_structure.id[idx], IDX::Interface(_)) {
                let mass_electron = mesh_structure.mass_electron(idx + 1);
                Some(SRHStatistics::new(temperature, mass_electron, thermal_velocity))
            } else {
                None
            }
        })
        .collect();
    Self {
        potential,
        mesh_structure,
        temperature,
        sor_relaxation_factor,
        red_indices,
        black_indices,
        convergence_threshold,
        max_iterations,
        electron_density_model: Box::new(BoltzmannApproximation::new(temperature)),
        donor_activation_model: DonorActivation::new(temperature),
        parallel_use,
        simulation_phase: SimulationPhase::Stress,
        interface_srh,
        previous_phase_occupation: vec![None; n],
        fermi_dirac: FermiDiracStatistics::new(temperature),
        stress_relief_time,
    }
}
```

- [ ] **Step 6: `src/solvers/poisson_solver.rs` 内の全テスト `PoissonSolver::new` 呼び出しを更新**

テスト内の全呼び出しに `, 0.0, 0.0` を追加。該当行（全部で約15箇所）:

```
行526: PoissonSolver::new(mesh, initial_potential, 300.0, 1.0, 1e-6, 1000, false)
行544: PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false)
行558: PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false)
行583: PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false)
行613: PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-10, 100_000, false)
行663: PoissonSolver::new(mesh, 0.0, 300.0, 1.0, f64::MAX, 1000, false)
行678: PoissonSolver::new(mesh, 0.0, 300.0, 1.0, -1.0, 123, false)
行703: PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1, false)
行735: PoissonSolver::new(mesh_no_charge, 0.0, 300.0, 1.0, 1e-6, 1, false)
行739: PoissonSolver::new(mesh_with_charge, 0.0, 300.0, 1.0, 1e-6, 1, false)
行763: PoissonSolver::new(mesh, uniform_pot, 300.0, 1.0, 1e-6, 1, false)
行785: PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1, false)
行807: PoissonSolver::new(mesh, initial_potential, 300.0, 1.0, 1e-6, 1000, false)
行824: PoissonSolver::new(mesh, initial_potential, 300.0, 1.0, 1e-6, 1000, false)
```

それぞれ末尾の `false)` を `false, 0.0, 0.0)` に変更する。

- [ ] **Step 7: `src/solvers/cv_solver.rs` 内のテスト `PoissonSolver::new` 呼び出しを更新**

```rust
// 行322 変更前
let poisson_solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-8, 100_000, false);
// 変更後
let poisson_solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-8, 100_000, false, 0.0, 0.0);

// 行343 変更前
let poisson_solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false);
// 変更後
let poisson_solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 0.0, 0.0);
```

- [ ] **Step 8: `src/main.rs` の呼び出しを更新**

```rust
// 変更前
let poisson_solver = PoissonSolver::new(
    mesh_structure,
    INITIAL_POTENTIAL,
    config.measurement.temperature.temperature,
    config.sim_settings.sor_relaxation_factor,
    config.sim_settings.convergence_criterion,
    config.sim_settings.max_iterations,
    config.sim_settings.parallel_use,
);
```

```rust
// 変更後
let poisson_solver = PoissonSolver::new(
    mesh_structure,
    INITIAL_POTENTIAL,
    config.measurement.temperature.temperature,
    config.sim_settings.sor_relaxation_factor,
    config.sim_settings.convergence_criterion,
    config.sim_settings.max_iterations,
    config.sim_settings.parallel_use,
    config.capture_cross_section.thermal_velocity,
    config.measurement.stress.stress_relief_time,
);
```

- [ ] **Step 9: テストを実行してパスを確認**

```bash
cargo test test_new_interface_srh_some_for_interface_nodes test_new_simulation_phase_is_stress 2>&1 | tail -10
```

Expected: `test ... ok`

- [ ] **Step 10: 全テストが通ることを確認**

```bash
cargo test 2>&1 | tail -20
```

Expected: 全テスト ok

- [ ] **Step 11: コミット**

```bash
git add src/solvers/poisson_solver.rs src/solvers/cv_solver.rs src/main.rs
git commit -m "feat: add SRH/FermiDirac fields to PoissonSolver, update new() signature"
```

---

## Task 3: `compute_occupation_probability` の実装（3フェーズ）

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

- [ ] **Step 1: テスト用ヘルパー `make_interface_mesh_with_states` を書く**

テストモジュール内（既存ヘルパーの直後）に追加:

```rust
use crate::mesh_builder::mesh_builder::{InterfaceStatesDistribution, InterfaceProperties};

fn make_interface_mesh_with_states(
    permittivity: f64,
    acceptor_dit_val: f64,
    donor_dit_val: f64,
) -> MeshStructure {
    // energy grid: 0.0, 0.5, 1.0 eV (Ec-Et)
    let et_grid = vec![0.0, 0.5, 1.0];
    let n = et_grid.len();
    MeshStructure {
        id: vec![
            IDX::Surface,
            IDX::Bulk(0),
            IDX::Interface(0),
            IDX::Bulk(1),
            IDX::Bottom,
        ],
        name: vec![
            "Surface".to_string(),
            "Bulk0".to_string(),
            "Interface".to_string(),
            "Bulk1".to_string(),
            "Bottom".to_string(),
        ],
        depth: vec![0.0, 1e-9, 2e-9, 3e-9, 4e-9],
        property_type: vec![
            PropertyType::Surface(SurfaceProperties {
                permittivity: 0.0,
                delta_conduction_band: 0.0,
                bandgap_energy: 1.0,
            }),
            PropertyType::Bulk(BulkProperties {
                mass_electron: 0.2,
                permittivity,
                delta_conduction_band: 0.0,
                donor_concentration: 0.0,
                energy_level_donor: 0.05,
                fixcharge_density: FixChargeDensity::Bulk(0.0),
                bandgap_energy: 1.0,
            }),
            PropertyType::Interface(InterfaceProperties {
                fixcharge_density: FixChargeDensity::Interface(0.0),
                interface_states: InterfaceStates::Distribution(InterfaceStatesDistribution {
                    id: 0,
                    potential: et_grid,
                    acceptor_dit: vec![acceptor_dit_val; n],
                    donor_dit: vec![donor_dit_val; n],
                    capture_cross_section: vec![1e-15; n],
                }),
            }),
            PropertyType::Bulk(BulkProperties {
                mass_electron: 0.2 * M_ELECTRON,
                permittivity,
                delta_conduction_band: 0.0,
                donor_concentration: 0.0,
                energy_level_donor: 0.05,
                fixcharge_density: FixChargeDensity::Bulk(0.0),
                bandgap_energy: 1.0,
            }),
            PropertyType::Bottom(BottomProperties {
                permittivity: 0.0,
                delta_conduction_band: 0.0,
                bandgap_energy: 1.0,
            }),
        ],
    }
}
```

- [ ] **Step 2: Stress フェーズのテストを書く**

```rust
// -----------------------------------------------------------------------
// compute_occupation_probability() — Stress フェーズ
// -----------------------------------------------------------------------

/// Stress フェーズ: φ_node = Et_grid[k] のとき f = 0.5
#[test]
fn test_compute_occupation_stress_at_fermi_level() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    // Interface は idx=2, et_grid = [0.0, 0.5, 1.0]
    let mut solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 0.0,
    );
    // φ_node = 0.5 → Et - Ef = 0.5 - 0.5 = 0.0 at k=1 → FD(0) = 0.5
    solver.potential.potential[2] = 0.5;
    let occ = solver.compute_occupation_probability(2);
    assert_eq!(occ.len(), 3);
    assert!(
        relative_eq!(occ[1], 0.5, epsilon = 1e-10),
        "f at Fermi level should be 0.5, got {}",
        occ[1]
    );
}

/// Stress フェーズ: Et >> Ef (φ_node - Et_grid[k] >> 0) のとき f ≈ 0
#[test]
fn test_compute_occupation_stress_trap_above_fermi() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    let mut solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 0.0,
    );
    // φ_node = 0.5, Et_grid[2] = 1.0 → φ - Et = -0.5 → FD(-0.5) ≈ 1.0
    // φ_node = 0.5, Et_grid[0] = 0.0 → φ - Et = 0.5 → FD(0.5) ≈ 0 (300K: 0.5eV >> kT≈0.026eV)
    solver.potential.potential[2] = 0.5;
    let occ = solver.compute_occupation_probability(2);
    assert!(
        occ[0] < 1e-5,
        "trap at Ec (well above Ef) should be nearly unoccupied, got {}",
        occ[0]
    );
    assert!(
        occ[2] > 1.0 - 1e-5,
        "trap at Ev (well below Ef) should be nearly occupied, got {}",
        occ[2]
    );
}

/// non-Interface ノードに対して空 Vec が返ること
#[test]
fn test_compute_occupation_returns_empty_for_non_interface() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    let solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 0.0,
    );
    let occ = solver.compute_occupation_probability(1); // Bulk node
    assert!(occ.is_empty());
}

// -----------------------------------------------------------------------
// compute_occupation_probability() — Relief フェーズ
// -----------------------------------------------------------------------

/// Relief フェーズ: f_prev=1.0, eff_emission=0.5, FD≈0 → f ≈ 0.5
#[test]
fn test_compute_occupation_relief_partial_emission() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    let thermal_velocity = 2.6e5_f64;
    let mut solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, thermal_velocity, 0.0,
    );
    // et_grid[1] = 0.5 eV のトラップに対して τ を計算し、t = τ を stress_relief_time に使う
    // τ = exp(0.5 * q/kT) / (v_th * σ * Nc)
    // eff_emission(t=τ) = 1 - exp(-1) ≈ 0.6321
    // f_prev = 1.0 (全て占有)、φ_node = 10.0 (高ポテンシャル → FD ≈ 0)
    // f_relief = 1.0 * (1 - 0.6321) + 0.0 = 0.3679 ≈ exp(-1)
    let srh = solver.interface_srh[2].as_ref().unwrap();
    let tau = srh.electron_emission_time(0.5, 1e-15);
    drop(srh);

    solver.stress_relief_time = tau;
    solver.simulation_phase = SimulationPhase::Relief;
    solver.previous_phase_occupation[2] = Some(vec![1.0, 1.0, 1.0]);
    solver.potential.potential[2] = 10.0; // φ >> Et → FD ≈ 0

    let occ = solver.compute_occupation_probability(2);
    let expected = (-1.0_f64).exp(); // ≈ 0.3679
    assert!(
        relative_eq!(occ[1], expected, max_relative = 1e-4),
        "Relief: f should be exp(-1) ≈ 0.3679, got {}",
        occ[1]
    );
}

/// Relief フェーズ: f_prev + FD が 1.0 を超える場合、1.0 にクランプされること
#[test]
fn test_compute_occupation_relief_clamped_to_one() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    let mut solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 1e-20, // 極小 t → eff_emission ≈ 0
    );
    solver.simulation_phase = SimulationPhase::Relief;
    solver.previous_phase_occupation[2] = Some(vec![0.8, 0.8, 0.8]);
    // φ_node = 0.0 → FD(0.0 - 0.0) = 0.5 at k=0
    // f ≈ 0.8*(1-0) + 0.5 = 1.3 → clamped to 1.0
    solver.potential.potential[2] = 0.0;
    let occ = solver.compute_occupation_probability(2);
    assert!(
        occ[0] <= 1.0,
        "occupation probability should not exceed 1.0, got {}",
        occ[0]
    );
    assert!(
        relative_eq!(occ[0], 1.0, epsilon = 1e-10),
        "should be clamped to 1.0, got {}",
        occ[0]
    );
}

// -----------------------------------------------------------------------
// compute_occupation_probability() — Measurement フェーズ
// -----------------------------------------------------------------------

/// Measurement フェーズ: previous_phase_occupation がそのまま返ること
#[test]
fn test_compute_occupation_measurement_returns_previous() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    let mut solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 0.0,
    );
    solver.simulation_phase = SimulationPhase::Measurement;
    let expected = vec![0.3, 0.6, 0.9];
    solver.previous_phase_occupation[2] = Some(expected.clone());
    solver.potential.potential[2] = 5.0; // ポテンシャルは無視されるべき

    let occ = solver.compute_occupation_probability(2);
    assert_eq!(occ, expected);
}
```

- [ ] **Step 3: テストを実行して失敗を確認**

```bash
cargo test test_compute_occupation 2>&1 | tail -20
```

Expected: コンパイルエラー（`compute_occupation_probability` 未定義）

- [ ] **Step 4: `compute_occupation_probability` を実装**

`PoissonSolver` の `impl` ブロック内（`solve_interface` の前）に追加:

```rust
fn compute_occupation_probability(&self, idx: usize) -> Vec<f64> {
    let dist = match self.mesh_structure.interface_states(idx) {
        Some(InterfaceStates::Distribution(d)) => d,
        _ => return Vec::new(),
    };
    let srh = match &self.interface_srh[idx] {
        Some(s) => s,
        None => return Vec::new(),
    };
    let phi_node = self.potential.potential[idx];

    dist.potential
        .iter()
        .enumerate()
        .map(|(k, &et)| match self.simulation_phase {
            SimulationPhase::Stress => self.fermi_dirac.fermi_dirac(phi_node - et),
            SimulationPhase::Relief => {
                let eff_emission = srh.effective_emission_coefficient(
                    self.stress_relief_time,
                    et,
                    dist.capture_cross_section[k],
                );
                let f_prev = self.previous_phase_occupation[idx]
                    .as_ref()
                    .map(|v| v[k])
                    .unwrap_or(0.0);
                let f = f_prev * (1.0 - eff_emission)
                    + self.fermi_dirac.fermi_dirac(phi_node - et);
                f.min(1.0)
            }
            SimulationPhase::Measurement => self.previous_phase_occupation[idx]
                .as_ref()
                .map(|v| v[k])
                .unwrap_or(0.0),
        })
        .collect()
}
```

また、`stress_relief_time` を `pub` にする（テストから設定するため）:

```rust
// 変更前
stress_relief_time: f64,
// 変更後
pub stress_relief_time: f64,
```

- [ ] **Step 5: テストを実行してパスを確認**

```bash
cargo test test_compute_occupation 2>&1 | tail -20
```

Expected: 全テスト ok

- [ ] **Step 6: 全テストが通ることを確認**

```bash
cargo test 2>&1 | tail -20
```

Expected: 全テスト ok

- [ ] **Step 7: コミット**

```bash
git add src/solvers/poisson_solver.rs
git commit -m "feat: implement compute_occupation_probability (Stress/Relief/Measurement)"
```

---

## Task 4: `set_simulation_phase` の実装

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

- [ ] **Step 1: 失敗するテストを書く**

```rust
// -----------------------------------------------------------------------
// set_simulation_phase()
// -----------------------------------------------------------------------

/// set_simulation_phase を呼ぶと simulation_phase が更新されること
#[test]
fn test_set_simulation_phase_updates_phase() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    let mut solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 0.0,
    );
    assert_eq!(solver.simulation_phase, SimulationPhase::Stress);
    solver.set_simulation_phase(SimulationPhase::Relief);
    assert_eq!(solver.simulation_phase, SimulationPhase::Relief);
}

/// set_simulation_phase 呼び出し前の interface_occupation が
/// previous_phase_occupation にコピーされること
#[test]
fn test_set_simulation_phase_copies_occupation_to_previous() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    let mut solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 0.0,
    );
    let expected = vec![0.1, 0.5, 0.9];
    solver.potential.interface_occupation[2] = Some(expected.clone());

    solver.set_simulation_phase(SimulationPhase::Relief);

    assert_eq!(
        solver.previous_phase_occupation[2].as_ref().unwrap(),
        &expected
    );
}
```

- [ ] **Step 2: テストを実行して失敗を確認**

```bash
cargo test test_set_simulation_phase 2>&1 | tail -10
```

Expected: コンパイルエラー（`set_simulation_phase` 未定義）

- [ ] **Step 3: `set_simulation_phase` を実装**

`PoissonSolver` の `impl` ブロック（`set_temperature` の近く）に追加:

```rust
pub fn set_simulation_phase(&mut self, phase: SimulationPhase) {
    self.previous_phase_occupation = self.potential.interface_occupation.clone();
    self.simulation_phase = phase;
}
```

- [ ] **Step 4: テストを実行してパスを確認**

```bash
cargo test test_set_simulation_phase 2>&1 | tail -10
```

Expected: `test ... ok`

- [ ] **Step 5: 全テストが通ることを確認**

```bash
cargo test 2>&1 | tail -20
```

- [ ] **Step 6: コミット**

```bash
git add src/solvers/poisson_solver.rs
git commit -m "feat: implement set_simulation_phase"
```

---

## Task 5: `compute_qit_density` と `solve_interface` の更新

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

- [ ] **Step 1: 失敗するテストを書く**

```rust
// -----------------------------------------------------------------------
// solve_interface() — Dit あり
// -----------------------------------------------------------------------

/// acceptor-like Dit があるとき、solve_interface の delta_potential が
/// Dit なしより大きくなること（負電荷 → 界面ポテンシャル上昇方向）
#[test]
fn test_solve_interface_acceptor_dit_increases_delta() {
    let eps = 10.0 * EPSILON_0;

    // Dit なし
    let mesh_no_dit = make_interface_mesh(eps, 0.0);
    let mut s0 = PoissonSolver::new(mesh_no_dit, 0.0, 300.0, 1.0, 1e-6, 1, false, 0.0, 0.0);
    s0.potential.potential[1] = 0.3;
    s0.potential.potential[2] = 0.0;
    s0.potential.potential[3] = 0.3;
    let delta_no_dit = s0.solve_interface(2);

    // acceptor-like Dit あり、φ_node = 0.0 → FD(0 - 0) = 0.5 at Et=0
    // Qit = -Dit * f * dE < 0 → 正電荷効果 → delta が小さくなる (or larger?)
    // acceptor占有 → 負電荷 → 分子が増加 → delta_potential 増加
    let mesh_with_dit = make_interface_mesh_with_states(eps, 1e20, 0.0);
    let mut s1 = PoissonSolver::new(
        mesh_with_dit, 0.0, 300.0, 1.0, 1e-6, 1, false, 0.0, 0.0,
    );
    s1.potential.potential[1] = 0.3;
    s1.potential.potential[2] = 0.0;
    s1.potential.potential[3] = 0.3;
    let delta_with_dit = s1.solve_interface(2);

    // acceptor-like 占有 → 負電荷 → Q_ELECTRON * (fixcharge + qit) が減少 →
    // 分子増加 → delta 増加
    assert!(
        delta_with_dit > delta_no_dit,
        "acceptor Dit should increase delta: {} vs {}",
        delta_with_dit,
        delta_no_dit
    );
}
```

- [ ] **Step 2: テストを実行して失敗を確認**

```bash
cargo test test_solve_interface_acceptor_dit_increases_delta 2>&1 | tail -10
```

Expected: FAIL（Dit が考慮されていないので delta が変わらない）

- [ ] **Step 3: `compute_qit_density` を実装**

`PoissonSolver` の `impl` ブロック内（`compute_occupation_probability` の後）に追加:

```rust
fn compute_qit_density(&self, idx: usize) -> f64 {
    let dist = match self.mesh_structure.interface_states(idx) {
        Some(InterfaceStates::Distribution(d)) => d,
        _ => return 0.0,
    };
    let n = dist.potential.len();
    if n == 0 {
        return 0.0;
    }
    let occ = self.compute_occupation_probability(idx);
    occ.iter().enumerate().map(|(k, &f)| {
        let de = if k + 1 < n {
            dist.potential[k + 1] - dist.potential[k]
        } else if n >= 2 {
            dist.potential[n - 1] - dist.potential[n - 2]
        } else {
            1.0
        };
        (-dist.acceptor_dit[k] * f + dist.donor_dit[k] * (1.0 - f)) * de
    }).sum()
}
```

- [ ] **Step 4: `solve_interface` を更新して Qit を加算**

```rust
// 変更前
fn solve_interface(&self, idx: usize) -> f64 {
    let upper_mesh_length = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
    let lower_mesh_length = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
    let c_upper = self.mesh_structure.permittivity(idx - 1) / upper_mesh_length;
    let c_lower = self.mesh_structure.permittivity(idx + 1) / lower_mesh_length;

    let fixcharge_density = match self.mesh_structure.fixcharge_density(idx) {
        FixChargeDensity::Interface(q) => q, // in 1/m^2
        _ => 0.0,
    };

    let delta_potential = (c_upper * self.potential.potential[idx - 1]
        + c_lower * self.potential.potential[idx + 1]
        - Q_ELECTRON * fixcharge_density)
        / (c_upper + c_lower)
        - self.potential.potential[idx];
    delta_potential
}
```

```rust
// 変更後
fn solve_interface(&self, idx: usize) -> f64 {
    let upper_mesh_length = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
    let lower_mesh_length = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
    let c_upper = self.mesh_structure.permittivity(idx - 1) / upper_mesh_length;
    let c_lower = self.mesh_structure.permittivity(idx + 1) / lower_mesh_length;

    let fixcharge_density = match self.mesh_structure.fixcharge_density(idx) {
        FixChargeDensity::Interface(q) => q, // in 1/m^2
        _ => 0.0,
    };

    let qit = self.compute_qit_density(idx);

    let delta_potential = (c_upper * self.potential.potential[idx - 1]
        + c_lower * self.potential.potential[idx + 1]
        - Q_ELECTRON * (fixcharge_density + qit))
        / (c_upper + c_lower)
        - self.potential.potential[idx];
    delta_potential
}
```

- [ ] **Step 5: テストを実行してパスを確認**

```bash
cargo test test_solve_interface_acceptor_dit_increases_delta 2>&1 | tail -10
```

Expected: `test ... ok`

- [ ] **Step 6: 全テストが通ることを確認**

```bash
cargo test 2>&1 | tail -20
```

- [ ] **Step 7: コミット**

```bash
git add src/solvers/poisson_solver.rs
git commit -m "feat: implement compute_qit_density and update solve_interface with Qit"
```

---

## Task 6: `calculate_interface_occupation`・`get_potential_profile`・`set_temperature` の更新

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

- [ ] **Step 1: 失敗するテストを書く**

```rust
// -----------------------------------------------------------------------
// calculate_interface_occupation() / get_potential_profile()
// -----------------------------------------------------------------------

/// get_potential_profile 後、interface ノードの interface_occupation が
/// Some(Vec<f64>) で埋まっていること
#[test]
fn test_get_potential_profile_fills_interface_occupation() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    let mut solver = PoissonSolver::new(
        mesh, 0.5, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 0.0,
    );
    solver.set_boundary_conditions(0.5, 0.1);

    let profile = solver.get_potential_profile();

    // idx=2 が Interface → Some(vec) であること
    assert!(
        profile.interface_occupation[2].is_some(),
        "Interface node should have Some occupation"
    );
    let occ = profile.interface_occupation[2].as_ref().unwrap();
    assert_eq!(occ.len(), 3, "occupation should have 3 energy points");
    for &f in occ {
        assert!(f >= 0.0 && f <= 1.0, "occupation must be in [0, 1], got {}", f);
    }

    // 非インターフェースノードは None であること
    assert!(profile.interface_occupation[0].is_none());
    assert!(profile.interface_occupation[1].is_none());
    assert!(profile.interface_occupation[3].is_none());
    assert!(profile.interface_occupation[4].is_none());
}

// -----------------------------------------------------------------------
// set_temperature() — interface_srh 伝播テスト
// -----------------------------------------------------------------------

/// set_temperature 後、interface_srh の温度も更新されること
/// （SRHStatistics に get_temperature が存在するので確認する）
#[test]
fn test_set_temperature_propagates_to_interface_srh() {
    let mesh = make_interface_mesh_with_states(10.0 * EPSILON_0, 1e16, 0.0);
    let mut solver = PoissonSolver::new(
        mesh, 0.0, 300.0, 1.0, 1e-6, 1000, false, 2.6e5, 0.0,
    );
    solver.set_temperature(400.0);

    let srh = solver.interface_srh[2].as_ref().unwrap();
    assert!(
        relative_eq!(srh.get_temperature(), 400.0, epsilon = 1e-10),
        "SRHStatistics temperature should be 400.0, got {}",
        srh.get_temperature()
    );
}
```

- [ ] **Step 2: テストを実行して失敗を確認**

```bash
cargo test test_get_potential_profile_fills_interface_occupation test_set_temperature_propagates_to_interface_srh 2>&1 | tail -15
```

Expected: FAIL

- [ ] **Step 3: `calculate_interface_occupation` を実装**

`PoissonSolver` の `impl` ブロック内（`calculate_ionized_donor_concentration` の後）に追加:

```rust
fn calculate_interface_occupation(&mut self) {
    for idx in 0..self.mesh_structure.id.len() {
        if matches!(self.mesh_structure.id[idx], IDX::Interface(_)) {
            let occ = self.compute_occupation_probability(idx);
            if !occ.is_empty() {
                self.potential.interface_occupation[idx] = Some(occ);
            }
        }
    }
}
```

- [ ] **Step 4: `get_potential_profile` に `calculate_interface_occupation` 呼び出しを追加**

```rust
// 変更前
pub fn get_potential_profile(&mut self) -> Potential {
    self.calculate_electron_density();
    self.calculate_ionized_donor_concentration();
    self.potential.clone()
}
```

```rust
// 変更後
pub fn get_potential_profile(&mut self) -> Potential {
    self.calculate_electron_density();
    self.calculate_ionized_donor_concentration();
    self.calculate_interface_occupation();
    self.potential.clone()
}
```

- [ ] **Step 5: `set_temperature` を更新して `interface_srh` と `fermi_dirac` に伝播**

```rust
// 変更前
pub fn set_temperature(&mut self, temperature: f64) {
    self.temperature = temperature;
    self.donor_activation_model.set_temperature(temperature);
    self.electron_density_model.set_temperature(temperature);
}
```

```rust
// 変更後
pub fn set_temperature(&mut self, temperature: f64) {
    self.temperature = temperature;
    self.donor_activation_model.set_temperature(temperature);
    self.electron_density_model.set_temperature(temperature);
    self.fermi_dirac.set_temperature(temperature);
    for srh in self.interface_srh.iter_mut().flatten() {
        srh.set_temperature(temperature);
    }
}
```

- [ ] **Step 6: テストを実行してパスを確認**

```bash
cargo test test_get_potential_profile_fills_interface_occupation test_set_temperature_propagates_to_interface_srh 2>&1 | tail -10
```

Expected: `test ... ok`

- [ ] **Step 7: 全テストが通ることを確認**

```bash
cargo test 2>&1 | tail -20
```

Expected: 全テスト ok

- [ ] **Step 8: コミット**

```bash
git add src/solvers/poisson_solver.rs
git commit -m "feat: implement calculate_interface_occupation and update get_potential_profile, set_temperature"
```

---

## Self-Review チェックリスト

- [x] **Spec coverage:**
  - `interface_occupation: Vec<Option<Vec<f64>>>` → Task 1 ✓
  - `SimulationPhase` enum → Task 1 ✓
  - `interface_srh`, `previous_phase_occupation`, `fermi_dirac`, `stress_relief_time` フィールド → Task 2 ✓
  - `compute_occupation_probability` (Stress/Relief/Measurement) → Task 3 ✓
  - `set_simulation_phase` → Task 4 ✓
  - `solve_interface` への Qit 反映 → Task 5 ✓
  - `calculate_interface_occupation` + `get_potential_profile` 更新 → Task 6 ✓
  - `set_temperature` の propagation → Task 6 ✓
  - `main.rs` 呼び出し更新 → Task 2 ✓

- [x] **Placeholder なし:** 全ステップにコードあり

- [x] **型の一貫性:**
  - `compute_occupation_probability(idx: usize) -> Vec<f64>` → Task 3 実装 / Task 5 呼び出しで一致
  - `interface_occupation: Vec<Option<Vec<f64>>>` → Task 1 定義 / Task 6 書き込みで一致
  - `SimulationPhase` → Task 1 定義 / Task 3・4 使用で一致
