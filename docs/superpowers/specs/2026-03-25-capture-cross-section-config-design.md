# Capture Cross-Section Config Design

**Date:** 2026-03-25
**Issue:** #52

## Overview

Add configuration support for capture cross-section (捕獲断面積) in `src/config/`. The capture cross-section supports two selectable models: a constant model and an energy-dependent (exponential) model. One model is configured per interface and is shared across both continuous and discrete interface states at that interface.

## Data Structures

### `src/config/capture_cross_section.rs` (new file)

```rust
#[derive(Debug)]
pub enum CaptureCrossSectionModel {
    // Constant model: σ = σ₀ [m²] (stored in SI units)
    Constant { sigma: f64 },
    // Energy-dependent model: σ(E) = σ_mid * exp((E - E_mid) / E_slope) [m²] (stored in SI units)
    EnergyDependent { sigma_mid: f64, e_mid: f64, e_slope: f64 },
}

#[derive(Debug)]
pub struct CaptureCrossSectionConfig {
    pub interface_id: Vec<u32>,
    pub model: Vec<CaptureCrossSectionModel>,
}
```

One entry per interface. A single model covers both continuous and discrete traps at that interface.

### `Configuration` struct update

Add the following field to `Configuration` in `src/config/configuration_builder.rs`, inserted immediately after `discrete_interface_states`:

```rust
pub capture_cross_section: CaptureCrossSectionConfig,
```

## CLI Input Flow

### Function signature

```rust
pub fn define_capture_cross_section(
    continuous: &ContinuousInterfaceStatesConfig,
    discrete: &DiscreteInterfaceStatesConfig,
) -> CaptureCrossSectionConfig
```

### Iteration strategy

Iterate over the union of `continuous.interface_id` and `discrete.interface_id` (deduplicated, sorted). For each interface in that union, prompt for the capture cross-section model regardless of which trap type is present.

### Prompt example

Model selection uses a `get_input` + `loop`/`match` pattern, matching the existing `get_discrete_state_type()` precedent in `interface_states.rs`:

```
Interface 0 between Layer 0 (Name: Al2O3) and Layer 1 (Name: GaN)
Select capture cross-section model for interface 0:
  Constant (c) or Energy-dependent (e): default is c
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

Numeric inputs use `get_parsed_input_with_default_nonnegative`.

### Unit conversion

User inputs sigma in cm². Sigma is a physical area, so the conversion factor is 1 cm² = 1e-4 m². Convert at config time using `crate::constants::units::CM_TO_M.powi(2)` (= 1e-4).

Note: `CM2_TO_M2` (= 1e4) is for inverse-area density (cm⁻² → m⁻²) and must NOT be used here.

`e_mid` and `e_slope` are in eV and stored as-is (no conversion needed).

### Sign convention for `e_slope`

In the energy-dependent model `σ(E) = σ_mid * exp((E - E_mid) / E_slope)`, a positive `e_slope` means the cross-section increases for energies above `E_mid` and decreases for energies below `E_mid`. This convention follows the formula from Issue #52. The sign is preserved as entered by the user.

### Call site

`define_capture_cross_section` is called in `ConfigurationBuilder::from_interactive()` immediately after `define_interface_states`.

## Files Changed

| File | Change |
|------|--------|
| `src/config/capture_cross_section.rs` | New file: `CaptureCrossSectionModel`, `CaptureCrossSectionConfig`, `define_capture_cross_section` |
| `src/config/mod.rs` | Add `pub mod capture_cross_section;` |
| `src/config/configuration_builder.rs` | Add `capture_cross_section` field to `Configuration` (after `discrete_interface_states`), call `define_capture_cross_section` in `from_interactive()` |

## Out of Scope

- Changes to `src/physics_equations/` (computation using the capture cross-section values is a separate task)
- Trait abstraction over capture cross-section models (not necessary given the enum approach)
