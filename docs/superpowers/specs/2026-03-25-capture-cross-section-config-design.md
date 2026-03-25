# Capture Cross-Section Config Design

**Date:** 2026-03-25
**Issue:** #52

## Overview

Add configuration support for capture cross-section (捕獲断面積) in `src/config/`. The capture cross-section supports two selectable models: a constant model and an energy-dependent (exponential) model. The config applies per-interface and covers both continuous and discrete interface states.

## Data Structures

### `src/config/capture_cross_section.rs` (new file)

```rust
pub enum CaptureCrossSectionModel {
    // Constant model: σ = σ₀ [cm²]
    Constant { sigma: f64 },
    // Energy-dependent model: σ(E) = σ_mid * exp((E - E_mid) / E_slope) [cm²]
    EnergyDependent { sigma_mid: f64, e_mid: f64, e_slope: f64 },
}

pub struct CaptureCrossSectionConfig {
    pub interface_id: Vec<u32>,
    pub model: Vec<CaptureCrossSectionModel>,
}
```

### `Configuration` struct update

Add the following field to `Configuration` in `src/config/configuration_builder.rs`:

```rust
pub capture_cross_section: CaptureCrossSectionConfig,
```

## CLI Input Flow

`define_capture_cross_section` takes `ContinuousInterfaceStatesConfig` and `DiscreteInterfaceStatesConfig` as arguments and iterates only over interfaces that have interface states configured.

```
Interface 0 between Layer 0 (Name: Al2O3) and Layer 1 (Name: GaN)
Select capture cross-section model for interface 0:
  (c) Constant model
  (e) Energy-dependent model
  default: c
> e
Enter sigma_mid (cm^2) for interface 0: default is 1e-16
>
Enter E_mid (eV) for interface 0: default is 0.5
>
Enter E_slope (eV) for interface 0: default is 0.1
>
```

For the constant model:
```
Enter sigma (cm^2) for interface 0: default is 1e-16
>
```

`define_capture_cross_section` is called in `ConfigurationBuilder::from_interactive()` immediately after `define_interface_states`.

## Files Changed

| File | Change |
|------|--------|
| `src/config/capture_cross_section.rs` | New file: `CaptureCrossSectionModel`, `CaptureCrossSectionConfig`, `define_capture_cross_section` |
| `src/config/mod.rs` | Add `pub mod capture_cross_section;` |
| `src/config/configuration_builder.rs` | Add `capture_cross_section` field to `Configuration`, call `define_capture_cross_section` in `from_interactive()` |

## Out of Scope

- Changes to `src/physics_equations/` (computation using the capture cross-section values is a separate task)
- Trait abstraction over capture cross-section models (not necessary given the enum approach)
