pub mod measurement;
pub mod sim_settings;
pub mod structure;

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::Paragraph,
};

/// A single editable form field.
pub enum Field {
    Text {
        label: String,
        value: String,
        cursor: usize, // byte index
        error: Option<String>,
    },
    Bool {
        label: String,
        value: bool,
    },
    Enum {
        label: String,
        options: Vec<&'static str>,
        idx: usize,
    },
}

impl Field {
    pub fn text(label: impl Into<String>, default: impl Into<String>) -> Self {
        let s = default.into();
        let cursor = s.len();
        Field::Text { label: label.into(), value: s, cursor, error: None }
    }

    pub fn bool_field(label: impl Into<String>, default: bool) -> Self {
        Field::Bool { label: label.into(), value: default }
    }

    pub fn enum_field(
        label: impl Into<String>,
        options: Vec<&'static str>,
        default_idx: usize,
    ) -> Self {
        Field::Enum { label: label.into(), options, idx: default_idx }
    }

    pub fn label(&self) -> &str {
        match self {
            Field::Text { label, .. } | Field::Bool { label, .. } | Field::Enum { label, .. } => {
                label
            }
        }
    }

    pub fn error(&self) -> Option<&str> {
        if let Field::Text { error, .. } = self { error.as_deref() } else { None }
    }

    pub fn set_error(&mut self, err: Option<String>) {
        if let Field::Text { error, .. } = self {
            *error = err;
        }
    }

    pub fn text_value(&self) -> &str {
        if let Field::Text { value, .. } = self { value } else { "" }
    }

    pub fn bool_value(&self) -> bool {
        if let Field::Bool { value, .. } = self { *value } else { false }
    }

    pub fn enum_idx(&self) -> usize {
        if let Field::Enum { idx, .. } = self { *idx } else { 0 }
    }

    /// Handle keyboard input. Returns true if consumed.
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match self {
            Field::Text { value, cursor, error, .. } => match key {
                KeyCode::Char(c) => {
                    value.insert(*cursor, c);
                    *cursor += c.len_utf8();
                    *error = None;
                    true
                }
                KeyCode::Backspace => {
                    if *cursor > 0 {
                        if let Some((idx, _)) = value[..*cursor].char_indices().last() {
                            value.remove(idx);
                            *cursor = idx;
                        }
                    }
                    true
                }
                KeyCode::Delete => {
                    if *cursor < value.len() {
                        value.remove(*cursor);
                    }
                    true
                }
                KeyCode::Left => {
                    if let Some((idx, _)) = value[..*cursor].char_indices().last() {
                        *cursor = idx;
                    }
                    true
                }
                KeyCode::Right => {
                    if *cursor < value.len() {
                        let c = value[*cursor..].chars().next().unwrap();
                        *cursor += c.len_utf8();
                    }
                    true
                }
                KeyCode::Home => {
                    *cursor = 0;
                    true
                }
                KeyCode::End => {
                    *cursor = value.len();
                    true
                }
                _ => false,
            },
            Field::Bool { value, .. } => match key {
                KeyCode::Char(' ') | KeyCode::Enter => {
                    *value = !*value;
                    true
                }
                _ => false,
            },
            Field::Enum { options, idx, .. } => match key {
                KeyCode::Left => {
                    *idx = if *idx == 0 { options.len() - 1 } else { *idx - 1 };
                    true
                }
                KeyCode::Right => {
                    *idx = (*idx + 1) % options.len();
                    true
                }
                _ => false,
            },
        }
    }
}

const LABEL_WIDTH: u16 = 32;

/// Render one field row (1 line) at the given rect.
pub fn render_field(frame: &mut Frame, field: &Field, focused: bool, area: Rect) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let chunks = Layout::horizontal([Constraint::Length(LABEL_WIDTH), Constraint::Min(0)])
        .split(area);

    let has_error = field.error().is_some();
    let label_style = if focused {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else if has_error {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Gray)
    };

    let label_text = format!("{:<w$}", format!("{}:", field.label()), w = LABEL_WIDTH as usize);
    frame.render_widget(Paragraph::new(label_text).style(label_style), chunks[0]);

    match field {
        Field::Text { value, cursor, error, .. } => {
            let style = if focused {
                Style::default().fg(Color::Yellow)
            } else if error.is_some() {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            let w = (chunks[1].width as usize).saturating_sub(2).max(1);
            // Determine visible window around cursor
            let before = &value[..*cursor];
            let before_chars = before.chars().count();
            let start_char = before_chars.saturating_sub(w);
            let all_chars: Vec<char> = value.chars().collect();
            let visible: String = all_chars
                .iter()
                .skip(start_char)
                .take(w)
                .collect();
            let padded = format!("{:<w$}", visible, w = w);
            let text = format!("[{}]", padded);
            frame.render_widget(Paragraph::new(text).style(style), chunks[1]);

            if focused {
                let col = (before_chars - start_char) as u16;
                let col = col.min(chunks[1].width.saturating_sub(2));
                frame.set_cursor_position((chunks[1].x + 1 + col, chunks[1].y));
            }
        }
        Field::Bool { value, .. } => {
            let style = if focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            frame.render_widget(
                Paragraph::new(if *value { "[x]" } else { "[ ]" }).style(style),
                chunks[1],
            );
        }
        Field::Enum { options, idx, .. } => {
            let style = if focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            let cur = options.get(*idx).copied().unwrap_or("?");
            frame.render_widget(
                Paragraph::new(format!("< {} >", cur)).style(style),
                chunks[1],
            );
        }
    }
}

/// Render an error hint line.
pub fn render_error_hint(frame: &mut Frame, msg: &str, area: Rect) {
    frame.render_widget(
        Paragraph::new(format!("  ^ {}", msg))
            .style(Style::default().fg(Color::Red)),
        area,
    );
}

/// Parse a field's text value as f64.
pub fn parse_f64(field: &mut Field, field_name: &str) -> Result<f64, String> {
    let s = field.text_value().trim().to_string();
    match s.parse::<f64>() {
        Ok(v) => {
            field.set_error(None);
            Ok(v)
        }
        Err(_) => {
            let msg = format!("{}: invalid number", field_name);
            field.set_error(Some(msg.clone()));
            Err(msg)
        }
    }
}

/// Parse a field's text value as f64 and validate.
pub fn parse_f64_validated<F: Fn(f64) -> bool>(
    field: &mut Field,
    field_name: &str,
    validate: F,
    err_msg: &str,
) -> Result<f64, String> {
    let v = parse_f64(field, field_name)?;
    if validate(v) {
        Ok(v)
    } else {
        let msg = format!("{}: {}", field_name, err_msg);
        field.set_error(Some(msg.clone()));
        Err(msg)
    }
}

/// Parse a field's text value as usize.
pub fn parse_usize(field: &mut Field, field_name: &str) -> Result<usize, String> {
    let s = field.text_value().trim().to_string();
    match s.parse::<usize>() {
        Ok(v) => {
            field.set_error(None);
            Ok(v)
        }
        Err(_) => {
            let msg = format!("{}: must be a non-negative integer", field_name);
            field.set_error(Some(msg.clone()));
            Err(msg)
        }
    }
}
