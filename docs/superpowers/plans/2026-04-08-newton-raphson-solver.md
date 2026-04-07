# Newton-Raphson Poisson Solver Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** SORソルバーをNewton-Raphson法（三重対角ヤコビアン + Thomas algorithm）に置き換え、収束速度を劇的に改善する。

**Architecture:** 離散化されたPoisson方程式の残差ベクトルと三重対角ヤコビアンを構築し、Thomas algorithm（O(N)直接法）で線形系を解く。各ノード型（Bulk/Interface）に対して解析的ヤコビアン対角要素を計算する。CVSolverの`solve_cv()`でDCポテンシャルのスナップショットを保存・復元し、ウォームスタートの質を改善する。

**Tech Stack:** Rust, `rayon`（不要になるため使用削除）, `indicatif`, `approx`（テスト用）

---

## ファイルマップ

| ファイル | 変更内容 |
|---|---|
| `src/physics_equations/donor_activation.rs` | `ionized_donor_dphi()` 追加 |
| `src/solvers/poisson_solver.rs` | SOR削除、NRソルバー・ヤコビアン追加 |
| `src/solvers/cv_solver.rs` | `solve_cv()` にウォームスタート修正 |
| `src/config/sim_settings.rs` | `sor_relaxation_factor`・`parallel_use` 削除 |

---

## Task 1: `ionized_donor_dphi` の追加

**Files:**
- Modify: `src/physics_equations/donor_activation.rs`

### 背景

`ionized_donor_concentration` の式：
```
Nd+ = Nd / (1 + 2·exp(-φ·q/kBT))
```
解析微分（`x = 2·exp(-φ·q/kBT)` と置く）：
```
d(Nd+)/dφ = Nd · x · (q/kBT) / (1 + x)²
```

- [ ] **Step 1: 失敗するテストを書く**

`src/physics_equations/donor_activation.rs` の `#[cfg(test)]` ブロックに追加：

```rust
/// ionized_donor_dphi のテスト：有限差分との比較
#[test]
fn test_ionized_donor_dphi_matches_finite_difference() {
    let da = DonorActivation::new(300.0);
    let nd = 1e22_f64;
    let phi = 0.3_f64;
    let eps = 1e-7;
    let numerical = (da.ionized_donor_concentration(nd, phi + eps)
        - da.ionized_donor_concentration(nd, phi - eps))
        / (2.0 * eps);
    let analytical = da.ionized_donor_dphi(nd, phi);
    assert!(
        approx::relative_eq!(analytical, numerical, max_relative = 1e-4),
        "analytical={analytical}, numerical={numerical}"
    );
}

/// phi=0 でのチェック（x=2, 分子=Nd*2*(q/kBT)/9）
#[test]
fn test_ionized_donor_dphi_at_zero_phi() {
    use crate::constants::physics::{K_BOLTZMANN, Q_ELECTRON};
    let temp = 300.0;
    let da = DonorActivation::new(temp);
    let nd = 1e22_f64;
    let q_per_kbt = Q_ELECTRON / (K_BOLTZMANN * temp);
    // x=2, (1+x)^2=9
    let expected = nd * 2.0 * q_per_kbt / 9.0;
    let result = da.ionized_donor_dphi(nd, 0.0);
    assert!(
        approx::relative_eq!(result, expected, max_relative = 1e-10),
        "result={result}, expected={expected}"
    );
}

/// phi が大きい（完全電離域）では微分がほぼ0になること
#[test]
fn test_ionized_donor_dphi_large_phi_near_zero() {
    let da = DonorActivation::new(300.0);
    let result = da.ionized_donor_dphi(1e22, 5.0);
    assert!(result.abs() < 1e10, "should be near zero at large phi: {result}");
}
```

- [ ] **Step 2: テストが失敗することを確認**

```bash
cargo test test_ionized_donor_dphi 2>&1 | head -20
```
期待出力：`error[E0599]: no method named `ionized_donor_dphi``

- [ ] **Step 3: `ionized_donor_dphi` を実装**

`src/physics_equations/donor_activation.rs` の `impl DonorActivation` ブロックに追加（`ionized_donor_concentration` の直後）：

```rust
/// Derivative of ionized donor density with respect to potential phi (Ed - Ef in eV).
///
/// Formula: d(Nd+)/dphi = Nd * x * (q/kBT) / (1 + x)^2
/// where x = 2 * exp(-phi * q/kBT)
pub fn ionized_donor_dphi(&self, donor_concentration: f64, phi: f64) -> f64 {
    let x = 2.0 * (-phi * self.q_per_kbt).exp();
    let denom = 1.0 + x;
    donor_concentration * x * self.q_per_kbt / (denom * denom)
}
```

- [ ] **Step 4: テストが通ることを確認**

```bash
cargo test test_ionized_donor_dphi
```
期待出力：3テストすべて `ok`

- [ ] **Step 5: コミット**

```bash
git add src/physics_equations/donor_activation.rs
git commit -m "feat: add ionized_donor_dphi analytical derivative"
```

---

## Task 2: `thomas_solve` の実装

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

Thomas algorithm（前進消去 + 後退代入）で三重対角線形系 `J·δφ = rhs` を解く。
境界ノードは固定なので、内部ノードのみ（サイズ M = N-2）を対象とする。

> **Note:** Tasks 2–5 のテストコードは `PoissonSolver::new` の**現行7引数シグネチャ**を使う。Task 6 でシグネチャを5引数に変更したとき、これらのテストも合わせて修正する。

- [ ] **Step 1: 失敗するテストを書く**

`src/solvers/poisson_solver.rs` の `#[cfg(test)]` に追加：

```rust
/// thomas_solve: 既知の解を持つ三重対角系で検証
/// [2 -1  0] [x0]   [1]      解: x = [1, 1, 1]
/// [-1 2 -1] [x1] = [0]
/// [0 -1  2] [x2]   [1]
#[test]
fn test_thomas_solve_known_solution() {
    let lower = vec![0.0, -1.0, -1.0]; // lower[0] は未使用
    let diag  = vec![ 2.0,  2.0,  2.0];
    let upper = vec![-1.0, -1.0,  0.0]; // upper[M-1] は未使用
    let rhs   = vec![ 1.0,  0.0,  1.0];
    let x = PoissonSolver::thomas_solve(&lower, &diag, &upper, &rhs);
    for (i, &xi) in x.iter().enumerate() {
        assert!(
            approx::relative_eq!(xi, 1.0, epsilon = 1e-12),
            "x[{i}]={xi} != 1.0"
        );
    }
}

/// thomas_solve: 1要素の系
#[test]
fn test_thomas_solve_single_element() {
    let lower = vec![0.0];
    let diag  = vec![3.0];
    let upper = vec![0.0];
    let rhs   = vec![6.0];
    let x = PoissonSolver::thomas_solve(&lower, &diag, &upper, &rhs);
    assert!(approx::relative_eq!(x[0], 2.0, epsilon = 1e-12));
}

/// thomas_solve: 非対称の三重対角系
#[test]
fn test_thomas_solve_asymmetric() {
    // [3 1 0] [x0]   [5]      x0=1, x1=2, x2=3
    // [2 4 1] [x1] = [12]
    // [0 1 5] [x2]   [17]
    let lower = vec![0.0, 2.0, 1.0];
    let diag  = vec![3.0, 4.0, 5.0];
    let upper = vec![1.0, 1.0, 0.0];
    let rhs   = vec![5.0, 12.0, 17.0];
    let x = PoissonSolver::thomas_solve(&lower, &diag, &upper, &rhs);
    assert!(approx::relative_eq!(x[0], 1.0, epsilon = 1e-10));
    assert!(approx::relative_eq!(x[1], 2.0, epsilon = 1e-10));
    assert!(approx::relative_eq!(x[2], 3.0, epsilon = 1e-10));
}
```

- [ ] **Step 2: テストが失敗することを確認**

```bash
cargo test test_thomas_solve 2>&1 | head -20
```
期待出力：`error[E0599]: no method named `thomas_solve``

- [ ] **Step 3: `thomas_solve` を実装**

`src/solvers/poisson_solver.rs` の `impl PoissonSolver` に追加：

```rust
fn thomas_solve(lower: &[f64], diag: &[f64], upper: &[f64], rhs: &[f64]) -> Vec<f64> {
    let n = rhs.len();
    let mut d = diag.to_vec();
    let mut b = rhs.to_vec();

    // Forward sweep
    for i in 1..n {
        debug_assert!(d[i - 1].abs() > 0.0, "thomas_solve: zero pivot at {}", i - 1);
        let factor = lower[i] / d[i - 1];
        d[i] -= factor * upper[i - 1];
        b[i] -= factor * b[i - 1];
    }

    // Backward substitution
    let mut x = vec![0.0; n];
    debug_assert!(d[n - 1].abs() > 0.0, "thomas_solve: zero pivot at {}", n - 1);
    x[n - 1] = b[n - 1] / d[n - 1];
    for i in (0..n - 1).rev() {
        x[i] = (b[i] - upper[i] * x[i + 1]) / d[i];
    }

    x
}
```

- [ ] **Step 4: テストが通ることを確認**

```bash
cargo test test_thomas_solve
```
期待出力：3テストすべて `ok`

- [ ] **Step 5: コミット**

```bash
git add src/solvers/poisson_solver.rs
git commit -m "feat: implement Thomas algorithm for tridiagonal solve"
```

---

## Task 3: Jacobian 対角要素の実装

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

各内部ノードの `∂F_i/∂φ_i` を解析的に計算する。バルクノードと界面ノードで式が異なる。

### 式の整理

**バルクノード:**
```
diag[i] = h_u * h_l / (2 * ε_i) * dρ/dφ - 1
dρ/dφ = -q * (dNd+/dφ - dn/dφ)
dn/dφ = -n * q/kBT   (Boltzmann: n = Nc * exp(-φ_eff * q/kBT))
dNd+/dφ = ionized_donor_dphi(Nd, φ_eff - Ed)
φ_eff = potential[idx] + delta_conduction_band(idx)
```

**界面ノード:**
```
diag[i] = -q * dQit/dφ / (c_u + c_l) - 1
dQit/dφ = Σ_k (-acceptor[k] - donor[k]) * df_k/dφ * ΔE_k
df_k/dφ = -f_k * (1 - f_k) * q/kBT
f_k = compute_occupation_probability(idx)[k]
```

- [ ] **Step 1: 失敗するテストを書く**

`src/solvers/poisson_solver.rs` の `#[cfg(test)]` に追加：

```rust
/// compute_jacobian_diagonal: バルクノードを有限差分で検証
#[test]
fn test_compute_jacobian_diagonal_bulk_matches_finite_difference() {
    use crate::constants::physics::EPSILON_0;
    let eps_material = 10.0 * EPSILON_0;
    let mesh = make_simple_mesh(0.2, eps_material, 1e22, 0.0);
    let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-8, 100_000, false);
    // Set a non-trivial potential
    solver.potential.potential[1] = 0.5;

    let idx = 1;
    let analytical = solver.compute_jacobian_diagonal(idx);

    let eps = 1e-6;
    solver.potential.potential[idx] += eps;
    let f_plus = solver.compute_delta(idx);
    solver.potential.potential[idx] -= 2.0 * eps;
    let f_minus = solver.compute_delta(idx);
    solver.potential.potential[idx] += eps; // restore
    let numerical = (f_plus - f_minus) / (2.0 * eps);

    assert!(
        approx::relative_eq!(analytical, numerical, max_relative = 1e-4),
        "analytical={analytical}, numerical={numerical}"
    );
}

/// compute_jacobian_diagonal: bulk, φが大きい（ほぼ完全電離・電子ゼロ）→ -1 に近い
#[test]
fn test_compute_jacobian_diagonal_bulk_large_phi() {
    use crate::constants::physics::EPSILON_0;
    let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
    let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-8, 100_000, false);
    solver.potential.potential[1] = 5.0; // fully depleted
    let diag = solver.compute_jacobian_diagonal(1);
    // At large phi, dn/dphi ≈ 0, dNd+/dphi ≈ 0 → diag ≈ -1
    assert!(
        approx::relative_eq!(diag, -1.0, max_relative = 1e-3),
        "diag={diag} should be near -1 at large phi"
    );
}
```

- [ ] **Step 2: テストが失敗することを確認**

```bash
cargo test test_compute_jacobian_diagonal 2>&1 | head -20
```
期待出力：`error[E0599]: no method named `compute_jacobian_diagonal``

- [ ] **Step 3: `compute_dqit_dphi` と `compute_jacobian_diagonal` を実装**

`src/solvers/poisson_solver.rs` の `impl PoissonSolver` に追加（`compute_qit_density` の直後）：

```rust
fn compute_dqit_dphi(&self, idx: usize) -> f64 {
    let dist = match self.mesh_structure.interface_states(idx) {
        Some(InterfaceStates::Distribution(d)) => d,
        _ => return 0.0,
    };
    let n = dist.potential.len();
    if n == 0 {
        return 0.0;
    }
    let q_per_kbt = Q_ELECTRON / (K_BOLTZMANN * self.temperature);
    let occ = self.compute_occupation_probability(idx);
    occ.iter()
        .enumerate()
        .map(|(k, &f)| {
            let df_dphi = -f * (1.0 - f) * q_per_kbt;
            let de = if k + 1 < n {
                dist.potential[k + 1] - dist.potential[k]
            } else if n >= 2 {
                dist.potential[n - 1] - dist.potential[n - 2]
            } else {
                1.0
            };
            (-dist.acceptor_dit[k] - dist.donor_dit[k]) * df_dphi * de
        })
        .sum()
}

fn compute_jacobian_diagonal(&self, idx: usize) -> f64 {
    match self.mesh_structure.id[idx] {
        IDX::Bulk(_) => {
            let h_u = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
            let h_l = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
            let phi_eff = self.potential.potential[idx]
                + self.mesh_structure.delta_conduction_band(idx);
            let n_e = self.electron_density_model.electron_density(
                phi_eff,
                self.mesh_structure.mass_electron(idx),
            );
            let q_per_kbt = Q_ELECTRON / (K_BOLTZMANN * self.temperature);
            let dn_dphi = -n_e * q_per_kbt;
            let phi_donor =
                phi_eff - self.mesh_structure.energy_level_donor(idx);
            let dnd_dphi = self.donor_activation_model.ionized_donor_dphi(
                self.mesh_structure.donor_concentration(idx),
                phi_donor,
            );
            let drho_dphi = -Q_ELECTRON * (dnd_dphi - dn_dphi);
            h_u * h_l / (2.0 * self.mesh_structure.permittivity(idx)) * drho_dphi - 1.0
        }
        IDX::Interface(_) => {
            let h_u = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
            let h_l = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
            let c_u = self.mesh_structure.permittivity(idx - 1) / h_u;
            let c_l = self.mesh_structure.permittivity(idx + 1) / h_l;
            let dqit_dphi = self.compute_dqit_dphi(idx);
            -Q_ELECTRON * dqit_dphi / (c_u + c_l) - 1.0
        }
        IDX::Surface | IDX::Bottom => {
            panic!("compute_jacobian_diagonal: boundary nodes have no Jacobian diagonal")
        }
    }
}
```

- [ ] **Step 4: テストが通ることを確認**

```bash
cargo test test_compute_jacobian_diagonal
```
期待出力：2テストすべて `ok`

- [ ] **Step 5: コミット**

```bash
git add src/solvers/poisson_solver.rs
git commit -m "feat: add Jacobian diagonal computation for NR solver"
```

---

## Task 4: `build_residual` と `build_jacobian` の実装

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

内部ノード（1..N-2）の残差ベクトルと三重対角ヤコビアンを構築する。

### 上下対角の式

**バルクノード:**
```
sub_diag[i] = h_l / (h_u + h_l)
super_diag[i] = h_u / (h_u + h_l)
```

**界面ノード:**
```
c_u = permittivity(idx-1) / h_u
c_l = permittivity(idx+1) / h_l
sub_diag[i] = c_u / (c_u + c_l)
super_diag[i] = c_l / (c_u + c_l)
```

- [ ] **Step 1: 失敗するテストを書く**

```rust
/// build_residual: 境界条件が満足されたとき（φが解のとき）残差がゼロに近いこと
#[test]
fn test_build_residual_at_solution_is_near_zero() {
    use crate::constants::physics::EPSILON_0;
    let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 0.0, 0.0);
    // Nd=0, fixcharge=0 → 解は線形（境界条件を満たすφ）
    let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-8, 100_000, false);
    solver.set_boundary_conditions(1.0, 0.0);
    // 線形補間が解: φ[1] = 2/3, φ[2] = 1/3 (for depth=[0,1,2,3] nm, phi=[1,?,?,0])
    // But make_simple_mesh has 4 nodes: [0]=surface, [1]=bulk, [2]=bulk, [3]=bottom
    // depth = [0, 1e-9, 2e-9, 3e-9]
    // For zero charge: phi = 1.0 - (1.0/3e-9)*depth
    solver.potential.potential[1] = 2.0 / 3.0;
    solver.potential.potential[2] = 1.0 / 3.0;
    let residual = solver.build_residual();
    for (i, &r) in residual.iter().enumerate() {
        assert!(
            r.abs() < 1e-10,
            "residual[{i}]={r} should be near zero at solution"
        );
    }
}

/// build_jacobian: 三重対角行列の上下対角がバルクノードで正しいこと
#[test]
fn test_build_jacobian_off_diagonals_bulk() {
    use crate::constants::physics::EPSILON_0;
    let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
    let solver = PoissonSolver::new(mesh, 0.0, 300.0, 1.0, 1e-8, 100_000, false);
    // depth = [0, 1e-9, 2e-9, 3e-9], h_u=h_l=1e-9 for idx=1
    // sub=h_l/(h_u+h_l)=0.5, super=h_u/(h_u+h_l)=0.5
    let (lower, _diag, upper) = solver.build_jacobian();
    // lower[0] と upper[1] は未使用だが、有効なインデックスの値を検証
    // idx=1: i=0
    assert!(
        approx::relative_eq!(lower[0], 0.5, epsilon = 1e-12),
        "lower[0]={}", lower[0]
    );
    assert!(
        approx::relative_eq!(upper[0], 0.5, epsilon = 1e-12),
        "upper[0]={}", upper[0]
    );
}
```

- [ ] **Step 2: テストが失敗することを確認**

```bash
cargo test test_build_residual test_build_jacobian 2>&1 | head -20
```
期待出力：`error[E0599]: no method named `build_residual``

- [ ] **Step 3: `compute_off_diagonal`・`build_residual`・`build_jacobian` を実装**

`src/solvers/poisson_solver.rs` の `impl PoissonSolver` に追加：

```rust
fn compute_off_diagonal(&self, idx: usize) -> (f64, f64) {
    match self.mesh_structure.id[idx] {
        IDX::Bulk(_) => {
            let h_u = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
            let h_l = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
            (h_l / (h_u + h_l), h_u / (h_u + h_l))
        }
        IDX::Interface(_) => {
            let h_u = self.mesh_structure.depth[idx] - self.mesh_structure.depth[idx - 1];
            let h_l = self.mesh_structure.depth[idx + 1] - self.mesh_structure.depth[idx];
            let c_u = self.mesh_structure.permittivity(idx - 1) / h_u;
            let c_l = self.mesh_structure.permittivity(idx + 1) / h_l;
            (c_u / (c_u + c_l), c_l / (c_u + c_l))
        }
        IDX::Surface | IDX::Bottom => {
            panic!("compute_off_diagonal: boundary nodes have no off-diagonal")
        }
    }
}

fn build_residual(&self) -> Vec<f64> {
    let n = self.mesh_structure.id.len();
    (1..n - 1).map(|idx| self.compute_delta(idx)).collect()
}

fn build_jacobian(&self) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = self.mesh_structure.id.len();
    let m = n - 2;
    let mut lower = vec![0.0; m];
    let mut diag = vec![0.0; m];
    let mut upper = vec![0.0; m];
    for i in 0..m {
        let idx = i + 1;
        diag[i] = self.compute_jacobian_diagonal(idx);
        let (sub, sup) = self.compute_off_diagonal(idx);
        lower[i] = sub;
        upper[i] = sup;
    }
    (lower, diag, upper)
}
```

- [ ] **Step 4: テストが通ることを確認**

```bash
cargo test test_build_residual test_build_jacobian
```
期待出力：2テストすべて `ok`

- [ ] **Step 5: コミット**

```bash
git add src/solvers/poisson_solver.rs
git commit -m "feat: add build_residual and build_jacobian for NR solver"
```

---

## Task 5: `solve_poisson_with_newton` の実装

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

Newton-Raphson反復ループ（バックトラッキングダンピング付き）を実装する。

- [ ] **Step 1: 失敗するテストを書く**

```rust
/// solve_poisson_with_newton: ゼロ電荷メッシュで収束後の残差がゼロに近いこと
#[test]
fn test_solve_poisson_with_newton_converges_zero_charge() {
    use crate::constants::physics::EPSILON_0;
    let mesh = make_simple_mesh(0.0, 10.0 * EPSILON_0, 0.0, 0.0);
    let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1e-10, 1000);
    solver.set_boundary_conditions(1.0, 0.0);
    solver.solve_poisson_with_newton();
    let residual = solver.build_residual();
    for (i, &r) in residual.iter().enumerate() {
        assert!(r.abs() < 1e-9, "residual[{i}]={r}");
    }
}

/// solve_poisson_with_newton: バルク電荷あり（Nd=1e22）でも収束すること
#[test]
fn test_solve_poisson_with_newton_converges_with_bulk_charge() {
    use crate::constants::physics::EPSILON_0;
    let mesh = make_simple_mesh(0.2, 10.0 * EPSILON_0, 1e22, 0.0);
    let mut solver = PoissonSolver::new(mesh, 0.0, 300.0, 1e-8, 10_000);
    solver.set_boundary_conditions(1.0, 0.1);
    let iters = solver.solve_poisson_with_newton();
    assert!(iters < 100, "NR should converge in <100 iterations, got {iters}");
    let residual = solver.build_residual();
    let max_r = residual.iter().map(|r| r.abs()).fold(0.0_f64, f64::max);
    assert!(max_r < 1e-6, "max residual={max_r}");
}
```

- [ ] **Step 2: テストが失敗することを確認**

```bash
cargo test test_solve_poisson_with_newton 2>&1 | head -20
```
期待出力：`error[E0599]: no method named `solve_poisson_with_newton``

- [ ] **Step 3: `solve_poisson_with_newton` を実装**

`src/solvers/poisson_solver.rs` の `impl PoissonSolver` に追加：

```rust
fn solve_poisson_with_newton(&mut self) -> usize {
    let pb = ProgressBar::new(self.max_iterations as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
            .unwrap(),
    );

    let n = self.mesh_structure.id.len();
    let mut iter_count = 0;
    let mut max_delta = 0.0;

    for i in 1..=self.max_iterations {
        iter_count = i;

        let residual = self.build_residual();
        let (lower, diag, upper) = self.build_jacobian();
        let rhs: Vec<f64> = residual.iter().map(|&r| -r).collect();
        let delta_phi = Self::thomas_solve(&lower, &diag, &upper, &rhs);

        // Backtracking line search
        let residual_norm: f64 = residual.iter().map(|r| r.abs()).sum();
        let saved: Vec<f64> = self.potential.potential[1..n - 1].to_vec();
        let mut alpha = 1.0_f64;
        for _ in 0..10 {
            for (j, &d) in delta_phi.iter().enumerate() {
                self.potential.potential[j + 1] = saved[j] + alpha * d;
            }
            let new_norm: f64 = self.build_residual().iter().map(|r| r.abs()).sum();
            if new_norm <= residual_norm {
                break;
            }
            alpha /= 2.0;
        }
        // potential は既に alpha を適用済み

        max_delta = delta_phi.iter().map(|d| d.abs()).fold(0.0_f64, f64::max);
        pb.set_message(format!("max|δφ|={:.3e}", max_delta));
        pb.inc(1);

        if max_delta <= self.convergence_threshold {
            break;
        }
    }

    pb.set_position(iter_count as u64);
    if iter_count >= self.max_iterations {
        pb.finish_with_message(format!(
            "max|δφ|={:.3e}. Reached max iterations without convergence.",
            max_delta
        ));
    } else {
        pb.abandon_with_message(format!(
            "max|δφ|={:.3e}. Converged.",
            max_delta
        ));
    }

    iter_count
}
```

- [ ] **Step 4: テストが通ることを確認**

```bash
cargo test test_solve_poisson_with_newton
```
期待出力：2テストすべて `ok`

- [ ] **Step 5: コミット**

```bash
git add src/solvers/poisson_solver.rs
git commit -m "feat: implement Newton-Raphson Poisson solver with backtracking"
```

---

## Task 6: SOR を NR に切り替え・不要コードの削除

**Files:**
- Modify: `src/solvers/poisson_solver.rs`

`solve_poisson()` が `solve_poisson_with_newton()` を呼ぶように変更し、SOR関連のコードとフィールドを削除する。

- [ ] **Step 1: `solve_poisson` をNRに切り替える**

`solve_poisson` メソッド内の `solve_poisson_with_sor` の呼び出しを、`solve_poisson_with_newton` に変更する。

変更前（`solve_poisson` 本体）:
```rust
for i in 1..=self.max_iterations {
    iter_count = i;
    sum_delta_potential = self.solve_poisson_with_sor(self.parallel_use);
    pb.set_message(format!("Δ φ={:.3e}", sum_delta_potential));
    pb.inc(1);
    if sum_delta_potential <= self.convergence_threshold {
        break;
    }
}
// ... pb finish ...
iter_count
```

変更後（`solve_poisson` 本体 全体）:
```rust
pub fn solve_poisson(&mut self, time_step: f64) -> usize {
    self.set_time_step(time_step);
    self.build_f_floor_cache();
    self.solve_poisson_with_newton()
}
```

既存の `pb` 生成・管理コードは `solve_poisson_with_newton` に移動済みなので、`solve_poisson` には不要。

- [ ] **Step 2: SOR関連メソッドを削除する**

以下のメソッドを `src/solvers/poisson_solver.rs` から削除する：

- `fn solve_poisson_with_sor(&mut self, parallel_use: bool) -> f64`
- `fn solve_poisson_with_sor_parallel(&mut self) -> f64`
- `fn solve_poisson_with_single_thread(&mut self) -> f64`

- [ ] **Step 3: SOR関連フィールドを削除する**

`PoissonSolver` 構造体から以下を削除：
```rust
sor_relaxation_factor: f64,    // 削除
red_indices: Vec<usize>,       // 削除
black_indices: Vec<usize>,     // 削除
parallel_use: bool,            // 削除
```

`PoissonSolver::new()` のシグネチャから `sor_relaxation_factor: f64` と `parallel_use: bool` を削除する。

変更前:
```rust
pub fn new(
    mesh_structure: MeshStructure,
    initial_potential: f64,
    temperature: f64,
    sor_relaxation_factor: f64,
    convergence_threshold: f64,
    max_iterations: usize,
    parallel_use: bool,
) -> Self
```

変更後:
```rust
pub fn new(
    mesh_structure: MeshStructure,
    initial_potential: f64,
    temperature: f64,
    convergence_threshold: f64,
    max_iterations: usize,
) -> Self
```

`new()` 本体から `sor_relaxation_factor`, `red_indices`, `black_indices`, `parallel_use` の初期化コードを削除する。

- [ ] **Step 4: `rayon` の使用箇所を削除する**

`poisson_solver.rs` 冒頭の `use rayon::prelude::*;` を削除する。

- [ ] **Step 5: コンパイルエラーを修正する**

```bash
cargo build 2>&1 | grep "error"
```

エラーが出る箇所の `PoissonSolver::new()` 呼び出しから `sor_relaxation_factor` と `parallel_use` 引数を削除する。対象ファイル：`src/main.rs`、`src/solvers/cv_solver.rs` の `#[cfg(test)]`、`src/solvers/poisson_solver.rs` の `#[cfg(test)]`（Tasks 2–5 で追加したテストを含む）。

`main.rs`:
```rust
// 変更前
let poisson_solver = PoissonSolver::new(
    mesh_structure,
    INITIAL_POTENTIAL,
    config.measurement.temperature.temperature,
    config.sim_settings.sor_relaxation_factor,  // 削除
    config.sim_settings.convergence_criterion,
    config.sim_settings.max_iterations,
    config.sim_settings.parallel_use,           // 削除
);

// 変更後
let poisson_solver = PoissonSolver::new(
    mesh_structure,
    INITIAL_POTENTIAL,
    config.measurement.temperature.temperature,
    config.sim_settings.convergence_criterion,
    config.sim_settings.max_iterations,
);
```

テストファイル内の `PoissonSolver::new(...)` 呼び出しも同様に修正する（`sor_relaxation_factor` と `parallel_use` の引数を削除）。

- [ ] **Step 6: 全テストが通ることを確認**

```bash
cargo test
```
期待出力：全テスト `ok`、コンパイルエラーなし

- [ ] **Step 7: コミット**

```bash
git add src/solvers/poisson_solver.rs src/main.rs src/solvers/cv_solver.rs
git commit -m "refactor: replace SOR with Newton-Raphson solver, remove SOR fields"
```

---

## Task 7: SimSettings から SOR 設定を削除

**Files:**
- Modify: `src/config/sim_settings.rs`

`sor_relaxation_factor` と `parallel_use` のフィールドとユーザー入力プロンプトを削除する。

- [ ] **Step 1: `SimSettings` 構造体を修正**

```rust
// 変更前
pub struct SimSettings {
    pub sim_name: String,
    pub sor_relaxation_factor: f64,
    pub convergence_criterion: f64,
    pub max_iterations: usize,
    pub parallel_use: bool,
}

// 変更後
pub struct SimSettings {
    pub sim_name: String,
    pub convergence_criterion: f64,
    pub max_iterations: usize,
}
```

- [ ] **Step 2: `define_sim_settings()` を修正**

`src/config/sim_settings.rs` の `define_sim_settings()` から以下を削除：

```rust
// 削除する行
let sor_relaxation_factor: f64 =
    get_parsed_input_with_default("Enter the SOR relaxation factor. Default is 1.9: ", 1.9);
let parallel_use: bool =
    get_bool_input("Use parallel processing for Poisson solver? (y/n). Default is n: ");
```

`SimSettings` の構築から `sor_relaxation_factor` と `parallel_use` フィールドも削除する：

```rust
// 変更後
SimSettings {
    sim_name,
    convergence_criterion,
    max_iterations,
}
```

- [ ] **Step 3: コンパイルエラーがないことを確認**

```bash
cargo build 2>&1 | grep "error"
```
期待出力：エラーなし

- [ ] **Step 4: テストが通ることを確認**

```bash
cargo test
```
期待出力：全テスト `ok`

- [ ] **Step 5: コミット**

```bash
git add src/config/sim_settings.rs
git commit -m "refactor: remove sor_relaxation_factor and parallel_use from SimSettings"
```

---

## Task 8: CVSolver のウォームスタート修正

**Files:**
- Modify: `src/solvers/cv_solver.rs`

`solve_cv()` でDCポテンシャルのスナップショットを保存し、AC解後に復元することで、次のDCステップのウォームスタートを改善する。

- [ ] **Step 1: 失敗するテストを書く**

`src/solvers/cv_solver.rs` の `#[cfg(test)]` に追加：

```rust
/// solve_cv の後にポテンシャルがDC解に戻っていること
#[test]
fn test_solve_cv_restores_dc_potential() {
    let mut cv_solver = make_cv_solver(0.2, 1e22, 1.0, 0.1, 0.0, 1.0, 0.1, 0.02);
    let gate_voltage = 0.5;

    // DC solve
    cv_solver.poisson_solver.set_boundary_conditions(
        -gate_voltage + cv_solver.boundary_conditions.barrier_height,
        cv_solver.boundary_conditions.ec_ef_bottom,
    );
    cv_solver.poisson_solver.solve_poisson(0.0);
    let dc_potential = cv_solver.poisson_solver.potential.potential.clone();

    // solve_cv runs AC ±ΔV and should restore potential to dc_potential
    let _ = cv_solver.solve_cv(gate_voltage).unwrap();
    let after_potential = &cv_solver.poisson_solver.potential.potential;

    for (i, (&dc, &after)) in dc_potential.iter().zip(after_potential.iter()).enumerate() {
        assert!(
            approx::relative_eq!(dc, after, epsilon = 1e-12),
            "potential[{i}] after solve_cv: dc={dc}, after={after}"
        );
    }
}
```

- [ ] **Step 2: テストが失敗することを確認**

```bash
cargo test test_solve_cv_restores_dc_potential 2>&1 | tail -10
```
期待出力：`FAILED`（現在は復元しないため）

- [ ] **Step 3: `solve_cv` を修正**

`src/solvers/cv_solver.rs` の `solve_cv` メソッドを以下に変更：

```rust
fn solve_cv(&mut self, gate_voltage: f64) -> anyhow::Result<f64> {
    let dc_snapshot = self.poisson_solver.potential.potential.clone();

    let electron_density_vg_plus_ac =
        self.electron_density_at_vg(gate_voltage + self.measurement.ac_voltage, 0.0);

    self.poisson_solver.potential.potential = dc_snapshot.clone();

    let electron_density_vg_minus_ac =
        self.electron_density_at_vg(gate_voltage - self.measurement.ac_voltage, 0.0);

    self.poisson_solver.potential.potential = dc_snapshot;

    let capacitance = Q_ELECTRON
        * (electron_density_vg_plus_ac - electron_density_vg_minus_ac)
        / (2.0 * self.measurement.ac_voltage);

    Ok(capacitance)
}
```

- [ ] **Step 4: テストが通ることを確認**

```bash
cargo test test_solve_cv_restores_dc_potential
```
期待出力：`ok`

- [ ] **Step 5: 全テストが通ることを確認**

```bash
cargo test
```
期待出力：全テスト `ok`

- [ ] **Step 6: コミット**

```bash
git add src/solvers/cv_solver.rs
git commit -m "fix: restore DC potential after solve_cv for better warm-starting"
```

---

## 最終確認

- [ ] `cargo clippy` でwarningがないことを確認

```bash
cargo clippy 2>&1 | grep "^error"
```

- [ ] `cargo fmt` でフォーマットを整える

```bash
cargo fmt
git add -u
git commit -m "style: cargo fmt"
```
