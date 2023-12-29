use dust_lang::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{
    borrow::Cow,
    fmt::Display,
    fs::File,
    io::{self, Write},
    path::PathBuf,
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

use super::Action;

pub struct Editor<'a> {
    current: usize,
    buffers: Vec<Buffer<'a>>,
    message: Option<Cow<'static, str>>,
}

impl<'a> Editor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            current: 0,
            buffers: Vec::new(),
            message: None,
        })
    }

    pub fn current_buffer(&self) -> &Buffer {
        &self.buffers[self.current]
    }

    pub fn add_buffer(&mut self, buffer: Buffer<'a>) {
        self.buffers.push(buffer);
    }

    pub fn run(&mut self, frame: &mut Frame, areas: &[Rect]) -> Option<Action> {
        let buffer = &self.buffers[self.current];
        let textarea = &buffer.textarea;
        let widget = textarea.widget();

        frame.render_widget(widget, areas[0]);

        // Render status line
        let modified = if buffer.modified { " [modified]" } else { "" };
        let slot = format!("[{}/{}]", self.current + 1, self.buffers.len());
        let path_text = if let Some(path) = &buffer.path {
            format!(" {}{} ", path.display(), modified)
        } else {
            "scratch".to_string()
        };
        let (row, col) = textarea.cursor();
        let cursor = format!("({},{})", row + 1, col + 1);

        let status_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(slot.len() as u16),
                    Constraint::Min(1),
                    Constraint::Length(cursor.len() as u16),
                ]
                .as_ref(),
            )
            .split(areas[1]);
        let status_style = Style::default().add_modifier(Modifier::REVERSED);
        frame.render_widget(Paragraph::new(slot).style(status_style), status_chunks[0]);
        frame.render_widget(
            Paragraph::new(path_text).style(status_style),
            status_chunks[1],
        );
        frame.render_widget(Paragraph::new(cursor).style(status_style), status_chunks[2]);

        // Render message at bottom
        let message = if let Some(message) = self.message.take() {
            Line::from(Span::raw(message))
        } else {
            Line::from(vec![
                Span::raw("Press "),
                Span::styled("^Q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to quit, "),
                Span::styled("^S", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to save, "),
                Span::styled("^G", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to search, "),
                Span::styled("^T", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to switch buffer "),
                Span::styled("^R", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to run"),
            ])
        };
        frame.render_widget(Paragraph::new(message), areas[2]);

        match crossterm::event::read().unwrap().into() {
            Input {
                key: Key::Char('r'),
                ctrl: true,
                ..
            } => return Some(Action::Submit),
            Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => return Some(Action::Quit),
            Input {
                key: Key::Char('t'),
                ctrl: true,
                ..
            } => {
                self.current = (self.current + 1) % self.buffers.len();
                self.message = Some(format!("Switched to buffer #{}", self.current + 1).into());
            }
            Input {
                key: Key::Char('s'),
                ctrl: true,
                ..
            } => {
                self.buffers[self.current].save().unwrap();
                self.message = Some("Saved!".into());
            }
            input => {
                let buffer = &mut self.buffers[self.current];
                buffer.modified = buffer.textarea.input(input);
            }
        }

        None
    }
}

pub struct Buffer<'a> {
    textarea: TextArea<'a>,
    path: Option<PathBuf>,
    modified: bool,
}

impl<'a> Buffer<'a> {
    pub fn new(content: String) -> Result<Self> {
        let mut textarea = TextArea::new(content.lines().map(|line| line.to_string()).collect());

        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));

        Ok(Self {
            textarea,
            path: None,
            modified: false,
        })
    }

    pub fn content(&self) -> String {
        self.textarea.lines().join("\n")
    }

    fn save(&mut self) -> io::Result<()> {
        if !self.modified {
            return Ok(());
        }

        let file = if let Some(path) = &self.path {
            File::create(path)?
        } else {
            File::create("/tmp/dust_buffer")?
        };

        let mut writer = io::BufWriter::new(file);

        for line in self.textarea.lines() {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        self.modified = false;

        Ok(())
    }
}
