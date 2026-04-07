# Newton-Raphson Poisson Solver — Design Spec

**Date:** 2026-04-08
**Branch:** 65-improve-compute-speed

## 背景と目的

GaN C-Vシミュレータのボトルネックは各 `solve_poisson()` 呼び出しでのSOR反復回数の多さ。
SOR（線形収束）をNewton-Raphson法（2次収束）に置き換えることで、同じ収束精度に達するまでの反復回数を劇的に削減する。

合わせて、CVソルバーにおけるウォームスタートの不整合（AC解後のポテンシャル状態の修正）も実施する。

---

## 変更スコープ

| ファイル | 変更内容 |
|---|---|
| `src/solvers/poisson_solver.rs` | SOR削除、NRソルバー追加、ヤコビアン計算追加 |
| `src/physics_equations/donor_activation.rs` | `ionized_donor_dphi()` メソッド追加 |
| `src/solvers/cv_solver.rs` | `solve_cv()` にウォームスタート修正 |

---

## 1. PoissonSolverの構造変更

### 削除するメソッド

- `solve_poisson_with_sor(&mut self, parallel_use: bool) -> f64`
- `solve_poisson_with_sor_parallel(&mut self) -> f64`
- `solve_poisson_with_single_thread(&mut self) -> f64`

### 削除するフィールド

- `parallel_use: bool`
- `red_indices: Vec<usize>`
- `black_indices: Vec<usize>`
- `sor_relaxation_factor: f64`

`PoissonSolver::new()` の引数から `sor_relaxation_factor` と `parallel_use` を削除する。
`main.rs` / テストコードの呼び出し箇所も合わせて修正する。

### 追加するメソッド

```rust
fn solve_poisson_with_newton(&mut self) -> f64
fn build_residual(&self) -> Vec<f64>
fn build_jacobian(&self) -> (Vec<f64>, Vec<f64>, Vec<f64>)  // (lower, diag, upper)
fn compute_jacobian_diagonal(&self, idx: usize) -> f64
fn thomas_solve(lower: &[f64], diag: &[f64], upper: &[f64], rhs: &[f64]) -> Vec<f64>
```

### solve_poisson_with_newton の処理フロー

```
loop (最大 max_iterations 回):
  1. F = build_residual()            // 内部ノード(1..N-2)の残差ベクトル
  2. (lo, d, up) = build_jacobian()  // 三重対角ヤコビアン
  3. delta_phi = thomas_solve(lo, d, up, -F)
  4. ダンピング: α = 1.0
     while ||F(φ + α·delta_phi)|| > ||F(φ)||:
       α /= 2  (最大10回)
  5. φ[1..N-2] += α * delta_phi
  6. convergence_check: ||delta_phi||_∞ < convergence_threshold → break
```

反復回数を返す（既存インターフェース `solve_poisson() -> usize` を維持）。

---

## 2. 残差ベクトル

内部ノード `i ∈ 1..N-2` に対して：

```
F[i] = compute_delta(i)   // 既存メソッドを再利用
```

`compute_delta` は既に `solve_bulk`/`solve_interface` を呼んでおり、そのまま残差として使用できる。

---

## 3. ヤコビアン行列（三重対角）

内部ノード数を `M = N - 2` とする。`i` は内部インデックス（元の配列インデックス `idx = i + 1`）。

### 上下対角（バルク・界面共通パターンが異なる）

**バルクノード `idx`:**
```
h_u = depth[idx] - depth[idx-1]
h_l = depth[idx+1] - depth[idx]
sub_diag[i]   = h_l / (h_u + h_l)
super_diag[i] = h_u / (h_u + h_l)
```

**界面ノード `idx`:**
```
c_u = permittivity(idx-1) / (depth[idx] - depth[idx-1])
c_l = permittivity(idx+1) / (depth[idx+1] - depth[idx])
sub_diag[i]   = c_u / (c_u + c_l)
super_diag[i] = c_l / (c_u + c_l)
```

### 対角要素 `compute_jacobian_diagonal(idx)`

**バルクノード `idx`:**
```
h_u = depth[idx] - depth[idx-1]
h_l = depth[idx+1] - depth[idx]
dn_dphi     = -electron_density(phi) * q_per_kbt
dNd_dphi    = donor_activation.ionized_donor_dphi(Nd, phi - Ed)  // 要追加
drho_dphi   = -Q_ELECTRON * (dNd_dphi - dn_dphi)
diag[i] = h_u * h_l / (2.0 * permittivity(idx)) * drho_dphi - 1.0
```

**界面ノード `idx`:**
```
c_u = permittivity(idx-1) / h_u
c_l = permittivity(idx+1) / h_l
dQit_dphi = Σ_k (-acceptor[k] - donor[k]) * df_k_dphi * dE_k
  where df_k_dphi = -f_k * (1 - f_k) * q/kBT
diag[i] = -Q_ELECTRON * dQit_dphi / (c_u + c_l) - 1.0
```

---

## 4. DonorActivationへの追加

`src/physics_equations/donor_activation.rs` に以下を追加する：

```rust
/// d(ionized_donor_concentration)/d(phi) の解析微分
/// phi = Ed - Ef in eV (= potential - energy_level_donor)
pub fn ionized_donor_dphi(&self, donor_concentration: f64, phi: f64) -> f64
```

現在の `ionized_donor_concentration` の式：
```
Nd+ = Nd / (1 + 2·exp(-φ·q/kBT))
```

解析微分：
```
x = 2·exp(-φ·q/kBT)
d(Nd+)/dφ = Nd · x · (q/kBT) / (1 + x)^2
```

実装では `x` を中間変数として計算し、それを利用して返す。

---

## 5. Thomas Algorithm

スタティック関数として実装する。ピボットがゼロになるケースは本問題の物理的制約上発生しないが、`debug_assert!` で確認する。

```rust
fn thomas_solve(lower: &[f64], diag: &[f64], upper: &[f64], rhs: &[f64]) -> Vec<f64>
```

- `lower[0]` は使用しない（境界条件で固定）
- `upper[M-1]` は使用しない（境界条件で固定）

---

## 6. ウォームスタート修正（CVSolver）

`CVSolver::solve_cv()` を修正する：

```rust
fn solve_cv(&mut self, gate_voltage: f64) -> anyhow::Result<f64> {
    let dc_snapshot = self.poisson_solver.potential.potential.clone();

    let n_plus = self.electron_density_at_vg(gate_voltage + self.measurement.ac_voltage, 0.0);

    self.poisson_solver.potential.potential = dc_snapshot.clone();

    let n_minus = self.electron_density_at_vg(gate_voltage - self.measurement.ac_voltage, 0.0);

    self.poisson_solver.potential.potential = dc_snapshot;

    let capacitance = Q_ELECTRON * (n_plus - n_minus) / (2.0 * self.measurement.ac_voltage);
    Ok(capacitance)
}
```

---

## 7. config変更

`src/config/sim_settings.rs` の `SimSettings` 構造体から `sor_relaxation_factor` と `parallel_use` フィールドを削除する。
`define_sim_settings()` 内の対応するユーザー入力プロンプト（`get_parsed_input_with_default` / `get_bool_input` の呼び出し）も削除する。
`convergence_criterion` と `max_iterations` は継続使用。

---

## 収束判定

`convergence_threshold` の意味が変わる：
- **旧（SOR）：** Σ|Δφ| < threshold（全ノードのポテンシャル変化の総和）
- **新（NR）：** max|δφ| < threshold（最大ポテンシャル更新量）

ユーザー設定の `convergence_criterion` をそのまま `max|δφ|` の閾値として使用する。
既存の設定値（例: `1e-8`）で問題ないが、NRの方が厳しい収束に達しやすいため値の調整は任意。

---

## テスト戦略

- 既存の `PoissonSolver` / `CVSolver` のテストがそのまま通ることを確認
- `thomas_solve` のユニットテスト追加（既知の三重対角系で検証）
- `compute_jacobian_diagonal` のユニットテスト追加（有限差分との比較）
- NRとSORの解が同じ収束解に達することを確認するintegrationテスト
