# Ratatui TUI Design — GaN C-V Simulator

**Date:** 2026-04-12  
**Branch:** 18-ratatui  
**Status:** Approved

## Overview

Replace the current `println!`/`get_input()` CLI with a full-screen ratatui TUI. The simulation logic is unchanged; only the I/O layer is replaced.

## Screen Flow

```
StartScreen ──→ ConfigScreen ──→ RunScreen
                    ↑↓
               (navigate freely between sections)
```

### StartScreen

Two options:
- `New (interactive)` — open ConfigScreen with defaults
- `Load from JSON file` — file picker listing `~/.config/gan-cv-simulator/*.json`, then open ConfigScreen pre-filled

### ConfigScreen

Split pane. Left pane: section navigation. Right pane: form for the selected section.

**Left nav order:**
1. Sim Settings
2. Structure
3. Measurement
4. Fixed Charge
5. Interface States
6. Capture Cross Section
7. Mesh
8. Boundary Conditions
9. `[Run Simulation]` (action item at bottom)

### RunScreen

Three-pane layout:
- Left (narrow): run status (Running / Done / Error), points completed counter
- Center: current voltage, current capacitance, progress bar (points done / total)
- Right: real-time C-V ASCII chart, axes update as new points arrive

## Component Design

### Form Field Widgets

| Field type | Widget | Example field |
|---|---|---|
| String | Text input box | `sim_name` |
| Float / usize | Text input with inline validation | `thickness`, `max_iterations` |
| Bool | `[x]` / `[ ]` toggle | `parallel_use` |
| Enum | `< Semiconductor >` left/right toggle | `material_type` |
| Layer list | Selectable list + bottom edit form | Structure section |

### Structure Section (Layer Management)

Upper half: scrollable list of layers showing `id  name  type  thickness`.  
Lower half: edit form for the currently selected layer.  
Keys `a` / `d` add or delete layers. `Enter` focuses the edit form. `Esc` returns focus to the list.

### C-V ASCII Chart (RunScreen)

Rendered with ratatui's `Canvas` or `Chart` widget. X-axis: gate voltage range. Y-axis: capacitance (nF/cm²). Each completed measurement point is plotted as it arrives via channel.

## Architecture

### Thread Model

```
Main thread (ratatui event loop, ~60 fps)
  ├── Keyboard event handling (crossterm)
  ├── Screen rendering
  └── mpsc::Receiver<SimProgress>
            ↑
    Solver thread
      └── CVSolver::run() (unchanged logic)
```

`SimProgress` message:
```rust
enum SimProgress {
    Point { voltage: f64, capacitance: f64, index: usize, total: usize },
    Done,
    Error(String),
}
```

The solver thread sends `SimProgress` variants over a single `mpsc::Receiver<SimProgress>`. On `Error`, the TUI displays the message and allows the user to return to ConfigScreen.

### AppState

```rust
enum AppState {
    Start,
    Config(ConfigState),
    Running(RunState),
    Done(CVResult),
    Error(String),
}
```

### File Structure

```
src/
  tui/
    mod.rs                      ← App struct, AppState, main event loop
    start.rs                    ← StartScreen widget
    config.rs                   ← ConfigScreen (split pane, section routing)
    forms/
      mod.rs
      sim_settings.rs
      structure.rs              ← layer list + per-layer edit form
      measurement.rs
      fixed_charge.rs
      interface_states.rs
      capture_cross_section.rs
      mesh.rs
      boundary_conditions.rs
    run.rs                      ← RunScreen (3-pane)
    chart.rs                    ← C-V ASCII chart widget
  main.rs                       ← calls tui::run(), no more define_* calls
  config/                       ← structs and validation unchanged
  solvers/                      ← unchanged
  (utils.rs get_input helpers   ← no longer called; kept but unused)
```

## Key Bindings

| Key | Context | Action |
|---|---|---|
| `↑` / `↓` | Left nav | Move section selection |
| `↑` / `↓` | Form / Layer list | Move between fields / layers |
| `Tab` / `Shift+Tab` | Form | Next / previous field |
| `←` / `→` | Enum field | Cycle value |
| `Enter` | Nav | Enter selected section |
| `Enter` | Layer list | Open layer in edit form |
| `a` | Structure section | Add new layer |
| `d` | Structure section | Delete selected layer |
| `r` | ConfigScreen | Run simulation (available at any time; unvisited sections use defaults) |
| `Esc` | Edit form | Return to layer list |
| `Esc` | Section form | Return focus to left nav |
| `q` / `Ctrl+C` | Anywhere | Quit |

## Dependencies

**Add to `Cargo.toml`:**
```toml
ratatui = "0.29"
crossterm = "0.28"
```

**Remove from `Cargo.toml`:**
```toml
indicatif = "0.18.4"   # replaced by ratatui progress bar
```

## Refactoring Scope

- `src/config/config_source.rs` — `select_config_source()` replaced by StartScreen; file deleted or emptied
- `src/config/configuration_builder.rs` — `from_interactive()` replaced by ConfigScreen; method removed
- All `define_*` functions (`define_sim_settings`, `define_structure`, etc.) — removed; TUI forms build the structs directly
- `src/main.rs` — reduced to `tui::run()` call
- `src/utils.rs` — `get_input`, `get_bool_input`, `get_parsed_input*` no longer called (kept to avoid breaking any tests referencing them)

## Out of Scope

- Changing simulation physics or solver logic
- Adding new configuration parameters
- Changing output file formats (CSV, PNG)
- Removing `utils.rs` input helpers (may be cleaned up in a follow-up)
