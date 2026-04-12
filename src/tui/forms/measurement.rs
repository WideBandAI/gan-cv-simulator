use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
};

use crate::config::measurement::{Measurement, Stress, Temperature, Time, Voltage};
use crate::constants::units::MV_TO_V;
use super::{Field, parse_f64, parse_f64_validated, render_field};

pub struct MeasurementForm {
    pub fields: Vec<Field>,
    pub focused: usize,
}

impl Default for MeasurementForm {
    fn default() -> Self {
        Self {
            fields: vec![
                Field::text("Temperature (K)", "300"),
                Field::text("Voltage start (V)", "-2"),
                Field::text("Voltage end (V)", "2"),
                Field::text("Voltage step (V)", "0.1"),
                Field::text("AC voltage (mV)", "20"),
                Field::text("Measurement time (s)", "100"),
                Field::text("Stress voltage (V)", "0"),
                Field::text("Stress relief voltage (V)", "0"),
                Field::text("Stress relief time (s)", "0"),
            ],
            focused: 0,
        }
    }
}

impl MeasurementForm {
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

    pub fn build(&mut self) -> Result<Measurement, Vec<String>> {
        let mut errors = Vec::new();

        let temperature = parse_f64_validated(
            &mut self.fields[0], "Temperature",
            |v| *v > 0.0, "must be > 0",
        )
        .unwrap_or_else(|e| { errors.push(e); 300.0 });

        let v_start = parse_f64(&mut self.fields[1], "Voltage start")
            .unwrap_or_else(|e| { errors.push(e); 0.0 });
        let v_end = parse_f64(&mut self.fields[2], "Voltage end")
            .unwrap_or_else(|e| { errors.push(e); 0.0 });
        let v_step = parse_f64_validated(
            &mut self.fields[3], "Voltage step",
            |v| *v != 0.0, "must not be zero",
        )
        .unwrap_or_else(|e| { errors.push(e); 0.1 });

        let ac_mv = parse_f64_validated(
            &mut self.fields[4], "AC voltage",
            |v| *v > 0.0, "must be > 0",
        )
        .unwrap_or_else(|e| { errors.push(e); 20.0 });

        let meas_time = parse_f64_validated(
            &mut self.fields[5], "Measurement time",
            |v| *v >= 0.0, "must be >= 0",
        )
        .unwrap_or_else(|e| { errors.push(e); 100.0 });

        let stress_v = parse_f64(&mut self.fields[6], "Stress voltage")
            .unwrap_or_else(|e| { errors.push(e); 0.0 });
        let stress_rel_v = parse_f64(&mut self.fields[7], "Stress relief voltage")
            .unwrap_or_else(|e| { errors.push(e); 0.0 });
        let stress_rel_t = parse_f64_validated(
            &mut self.fields[8], "Stress relief time",
            |v| *v >= 0.0, "must be >= 0",
        )
        .unwrap_or_else(|e| { errors.push(e); 0.0 });

        if errors.is_empty() {
            Ok(Measurement {
                temperature: Temperature { temperature },
                voltage: Voltage { start: v_start, end: v_end, step: v_step },
                ac_voltage: ac_mv * MV_TO_V,
                time: Time { measurement_time: meas_time },
                stress: Stress {
                    stress_voltage: stress_v,
                    stress_relief_voltage: stress_rel_v,
                    stress_relief_time: stress_rel_t,
                },
            })
        } else {
            Err(errors)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Measurement")
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
