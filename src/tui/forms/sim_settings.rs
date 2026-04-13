use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::config::sim_settings::SimSettings;
use crate::utils::anti_traversal_filename;
use super::{Field, parse_f64_validated, parse_usize, render_field};

pub struct SimSettingsForm {
    pub fields: Vec<Field>,
    pub focused: usize,
}

impl Default for SimSettingsForm {
    fn default() -> Self {
        Self {
            fields: vec![
                Field::text("Simulation name", "my_simulation"),
                Field::text("SOR relaxation factor", "1.9"),
                Field::text("Convergence criterion (eV)", "0.000001"),
                Field::text("Max iterations", "100000"),
                Field::bool_field("Use parallel processing", false),
            ],
            focused: 0,
        }
    }
}

impl SimSettingsForm {
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Tab | KeyCode::Down => {
                self.focused = (self.focused + 1) % self.fields.len();
                true
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.focused =
                    (self.focused + self.fields.len() - 1) % self.fields.len();
                true
            }
            _ => {
                if let Some(f) = self.fields.get_mut(self.focused) {
                    f.handle_key(key)
                } else {
                    false
                }
            }
        }
    }

    pub fn build(&mut self) -> Result<SimSettings, Vec<String>> {
        let mut errors = Vec::new();

        // sim_name
        let raw = self.fields[0].text_value().trim().to_string();
        let sim_name = if raw.is_empty() {
            errors.push("Simulation name: must not be empty".into());
            String::new()
        } else if anti_traversal_filename(&raw).is_none()
            || !raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            errors.push("Simulation name: use only letters, digits, '-', '_', '.'".into());
            self.fields[0].set_error(Some("invalid characters".into()));
            String::new()
        } else {
            raw
        };

        let sor = parse_f64_validated(
            &mut self.fields[1], "SOR factor",
            |v| v > 0.0 && v <= 2.0,
            "must be in (0, 2]",
        )
        .unwrap_or_else(|e| { errors.push(e); 1.9 });

        let conv = parse_f64_validated(
            &mut self.fields[2], "Convergence criterion",
            |v| v > 0.0,
            "must be > 0",
        )
        .unwrap_or_else(|e| { errors.push(e); 1e-6 });

        let max_iter = parse_usize(&mut self.fields[3], "Max iterations")
            .unwrap_or_else(|e| { errors.push(e); 100_000 });

        let parallel = self.fields[4].bool_value();

        if errors.is_empty() {
            Ok(SimSettings {
                sim_name,
                sor_relaxation_factor: sor,
                convergence_criterion: conv,
                max_iterations: max_iter,
                parallel_use: parallel,
            })
        } else {
            Err(errors)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Simulation Settings")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let row_constraints: Vec<Constraint> =
            (0..self.fields.len()).map(|_| Constraint::Length(1)).collect();
        let rows = Layout::vertical(row_constraints).split(inner);

        for (i, (field, &row)) in self.fields.iter().zip(rows.iter()).enumerate() {
            render_field(frame, field, i == self.focused, row);
        }
    }
}
