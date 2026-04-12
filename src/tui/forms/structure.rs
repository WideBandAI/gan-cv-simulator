use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::config::structure::{DeviceStructure, MaterialType};
use crate::constants::physics::{EPSILON_0, M_ELECTRON};
use crate::constants::units::{NM_TO_M, PER_CM3_TO_PER_M3};
use super::{Field, render_field};

const MATTYPE_IDX_SEMI: usize = 0;
const MATTYPE_IDX_INS: usize = 1;

pub struct LayerForm {
    pub fields: Vec<Field>,
    pub focused_field: usize,
}

impl LayerForm {
    pub fn new_semiconductor(id: u32) -> Self {
        Self {
            fields: vec![
                Field::text("Name", format!("layer_{}", id)),
                Field::enum_field("Material type", vec!["Semiconductor", "Insulator"], MATTYPE_IDX_SEMI),
                Field::text("Thickness (nm)", "100"),
                Field::text("Relative permittivity", "9.5"),
                Field::text("Bandgap energy (eV)", "3.4"),
                Field::text("Delta conduction band (eV)", "0"),
                Field::text("Electron effective mass coeff", "0.22"),
                Field::text("Donor concentration (cm⁻³)", "1e17"),
                Field::text("Energy level donor Ec-Ed (eV)", "0.01"),
            ],
            focused_field: 0,
        }
    }

    pub fn new_insulator(id: u32) -> Self {
        Self {
            fields: vec![
                Field::text("Name", format!("layer_{}", id)),
                Field::enum_field("Material type", vec!["Semiconductor", "Insulator"], MATTYPE_IDX_INS),
                Field::text("Thickness (nm)", "5"),
                Field::text("Relative permittivity", "9.0"),
                Field::text("Bandgap energy (eV)", "6.5"),
                Field::text("Delta conduction band (eV)", "2.0"),
            ],
            focused_field: 0,
        }
    }

    pub fn material_type(&self) -> MaterialType {
        if self.fields[1].enum_idx() == MATTYPE_IDX_SEMI {
            MaterialType::Semiconductor
        } else {
            MaterialType::Insulator
        }
    }

    pub fn name(&self) -> String {
        self.fields[0].text_value().trim().to_string()
    }

    pub fn num_fields(&self) -> usize {
        self.fields.len()
    }

    /// Called when material_type field changes: rebuild semiconductor fields if needed.
    pub fn sync_material_type(&mut self, id: u32) {
        let new_type = self.material_type();
        let is_semi = new_type == MaterialType::Semiconductor;
        let has_semi_fields = self.fields.len() > 6;

        if is_semi && !has_semi_fields {
            // Add semiconductor-specific fields with defaults
            self.fields.push(Field::text("Electron effective mass coeff", "0.22"));
            self.fields.push(Field::text("Donor concentration (cm⁻³)", "1e17"));
            self.fields.push(Field::text("Energy level donor Ec-Ed (eV)", "0.01"));
        } else if !is_semi && has_semi_fields {
            // Remove semiconductor-specific fields
            self.fields.truncate(6);
            if self.focused_field >= self.fields.len() {
                self.focused_field = self.fields.len() - 1;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        let id = 0u32; // id used only for sync
        match key {
            KeyCode::Tab | KeyCode::Down => {
                self.focused_field = (self.focused_field + 1) % self.fields.len();
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.focused_field =
                    (self.focused_field + self.fields.len() - 1) % self.fields.len();
            }
            _ => {
                let was_mat_type = self.focused_field == 1;
                if let Some(f) = self.fields.get_mut(self.focused_field) {
                    f.handle_key(key);
                }
                if was_mat_type {
                    self.sync_material_type(id);
                }
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let constraints: Vec<Constraint> =
            (0..self.fields.len()).map(|_| Constraint::Length(1)).collect();
        let rows = Layout::vertical(constraints).split(area);
        for (i, (f, &row)) in self.fields.iter().zip(rows.iter()).enumerate() {
            render_field(frame, f, i == self.focused_field, row);
        }
    }
}

pub struct StructureForm {
    pub layers: Vec<LayerForm>,
    pub selected: usize,
    pub list_focused: bool, // true = list pane focused; false = edit form focused
}

impl Default for StructureForm {
    fn default() -> Self {
        Self {
            layers: vec![LayerForm::new_semiconductor(0)],
            selected: 0,
            list_focused: true,
        }
    }
}

impl StructureForm {
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        if self.list_focused {
            match key {
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    true
                }
                KeyCode::Down => {
                    if self.selected + 1 < self.layers.len() {
                        self.selected += 1;
                    }
                    true
                }
                KeyCode::Enter => {
                    self.list_focused = false;
                    true
                }
                KeyCode::Char('a') => {
                    let id = self.layers.len() as u32;
                    self.layers.push(LayerForm::new_semiconductor(id));
                    self.selected = self.layers.len() - 1;
                    true
                }
                KeyCode::Char('d') => {
                    if !self.layers.is_empty() {
                        self.layers.remove(self.selected);
                        if self.selected >= self.layers.len() && !self.layers.is_empty() {
                            self.selected = self.layers.len() - 1;
                        }
                    }
                    true
                }
                _ => false,
            }
        } else {
            match key {
                KeyCode::Esc => {
                    self.list_focused = true;
                    true
                }
                _ => {
                    if let Some(layer) = self.layers.get_mut(self.selected) {
                        layer.handle_key(key);
                    }
                    true
                }
            }
        }
    }

    pub fn build(&mut self) -> Result<DeviceStructure, Vec<String>> {
        use crate::constants::units::NM_TO_M;
        if self.layers.is_empty() {
            return Err(vec!["Structure: at least one layer required".into()]);
        }

        let mut errors = Vec::new();
        let n = self.layers.len();

        let mut id = vec![];
        let mut name = vec![];
        let mut material_type = vec![];
        let mut thickness = vec![];
        let mut mass_electron = vec![];
        let mut permittivity = vec![];
        let mut bandgap_energy = vec![];
        let mut delta_conduction_band = vec![];
        let mut donor_concentration = vec![];
        let mut energy_level_donor = vec![];

        for (i, layer) in self.layers.iter_mut().enumerate() {
            id.push(i as u32);
            name.push(layer.fields[0].text_value().trim().to_string());
            let mat = layer.material_type();
            material_type.push(mat);

            let t_nm = layer.fields[2].text_value().trim().parse::<f64>();
            match t_nm {
                Ok(v) if v > 0.0 => thickness.push(v * NM_TO_M),
                _ => {
                    errors.push(format!("Layer {}: thickness must be > 0", i));
                    thickness.push(0.0);
                }
            }

            let perm = layer.fields[3].text_value().trim().parse::<f64>();
            match perm {
                Ok(v) if v > 0.0 => permittivity.push(v * EPSILON_0),
                _ => {
                    errors.push(format!("Layer {}: permittivity must be > 0", i));
                    permittivity.push(EPSILON_0);
                }
            }

            let bg = layer.fields[4].text_value().trim().parse::<f64>();
            match bg {
                Ok(v) if v > 0.0 => bandgap_energy.push(v),
                _ => {
                    errors.push(format!("Layer {}: bandgap must be > 0", i));
                    bandgap_energy.push(3.4);
                }
            }

            // delta_conduction_band: last layer is always 0
            if i == n - 1 {
                delta_conduction_band.push(0.0);
            } else {
                match layer.fields[5].text_value().trim().parse::<f64>() {
                    Ok(v) => delta_conduction_band.push(v),
                    Err(_) => {
                        errors.push(format!("Layer {}: delta Ec invalid", i));
                        delta_conduction_band.push(0.0);
                    }
                }
            }

            if mat == MaterialType::Semiconductor {
                let me = layer.fields.get(6).map(|f| f.text_value().trim().parse::<f64>());
                match me {
                    Some(Ok(v)) if v > 0.0 => mass_electron.push(v * M_ELECTRON),
                    _ => {
                        errors.push(format!("Layer {}: effective mass must be > 0", i));
                        mass_electron.push(0.22 * M_ELECTRON);
                    }
                }

                let nd = layer.fields.get(7).map(|f| f.text_value().trim().parse::<f64>());
                match nd {
                    Some(Ok(v)) if v >= 0.0 => donor_concentration.push(v * PER_CM3_TO_PER_M3),
                    _ => {
                        errors.push(format!("Layer {}: donor concentration invalid", i));
                        donor_concentration.push(0.0);
                    }
                }

                let ed = layer.fields.get(8).map(|f| f.text_value().trim().parse::<f64>());
                match ed {
                    Some(Ok(v)) => energy_level_donor.push(v),
                    _ => {
                        errors.push(format!("Layer {}: energy level invalid", i));
                        energy_level_donor.push(0.01);
                    }
                }
            } else {
                mass_electron.push(0.0);
                donor_concentration.push(0.0);
                energy_level_donor.push(0.0);
            }
        }

        if errors.is_empty() {
            Ok(DeviceStructure {
                id,
                name,
                material_type,
                thickness,
                mass_electron,
                permittivity,
                bandgap_energy,
                delta_conduction_band,
                donor_concentration,
                energy_level_donor,
            })
        } else {
            Err(errors)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Structure  [a: add layer  d: delete layer  Enter: edit  Esc: back to list]")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let [list_area, edit_area] =
            Layout::vertical([Constraint::Min(5), Constraint::Min(0)]).areas(inner);

        // Layer list
        let list_block = Block::default()
            .borders(Borders::ALL)
            .title("Layers")
            .border_style(if self.list_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let items: Vec<ListItem> = self
            .layers
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let name = l.fields[0].text_value();
                let mat = if l.material_type() == MaterialType::Semiconductor { "Semi" } else { "Ins" };
                let t = l.fields[2].text_value();
                ListItem::new(Line::from(format!(
                    "{:>2}  {:<15} {:>5}  {} nm",
                    i, name, mat, t
                )))
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));

        let list = List::new(items)
            .block(list_block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, list_area, &mut list_state);

        // Edit form for selected layer
        if let Some(layer) = self.layers.get(self.selected) {
            let edit_block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Edit layer {}", self.selected))
                .border_style(if !self.list_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });
            let edit_inner = edit_block.inner(edit_area);
            frame.render_widget(edit_block, edit_area);
            layer.render(frame, edit_inner);
        }
    }
}
