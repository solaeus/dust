#![allow(clippy::disallowed_methods)]

mod block_table;

use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs, Widget, Wrap},
};

use crate::{
    instruction::OperandType,
    program::Program,
    prototype::Prototype,
    source::{Code, FileId, Source},
    syntax::{Syntax, tree::SyntaxTree},
};

use block_table::BlockTable;

pub struct Disassembler<'a> {
    program: &'a Program,
    source: &'a Source<'a>,
    syntax: &'a Syntax,
    constant_tags: &'a [OperandType],

    tabs: Vec<Tab<'a>>,

    state: TuiState,
    selection_state: SelectionState,
}

impl<'a> Disassembler<'a> {
    pub fn new(
        program: &'a Program,
        source: &'a Source,
        syntax: &'a Syntax,
        constant_tags: &'a [OperandType],
    ) -> Self {
        let mut tabs = Vec::with_capacity(source.file_count() + program.prototypes.len() + 1);

        for (file_id, file) in source.iter() {
            tabs.push(Tab::SourceFile {
                file_name: file.file_name(),
                file_id,
            });
        }

        tabs.push(Tab::Constants);

        for (index, prototype) in program.prototypes.iter().enumerate() {
            tabs.push(Tab::Prototype(PrototypeTab {
                id: index as u16,
                prototype,
            }));
        }

        Self {
            program,
            source,
            syntax,
            constant_tags,
            state: TuiState::Run,
            selection_state: SelectionState {
                current_tab: 0,
                tab_count: tabs.len(),
                current_row: None,
                row_count: 0,
            },
            tabs,
        }
    }

    pub fn disassemble(mut self) -> io::Result<()> {
        let mut terminal = ratatui::init();

        while self.state == TuiState::Run {
            let draw_result = terminal.draw(|frame| frame.render_widget(&mut self, frame.area()));

            if let Err(error) = draw_result {
                ratatui::restore();

                return Err(error);
            }

            self.handle_events()?;
        }

        ratatui::restore();

        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Right | KeyCode::Char('l') => {
                    self.selection_state.next_tab();
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.selection_state.previous_tab();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selection_state.previous_row();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.selection_state.next_row();
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.state = TuiState::Quit;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn draw_source_tab(
        &self,
        source_file: &Code,
        syntax_tree: &SyntaxTree,
        area: Rect,
        buffer: &mut Buffer,
    ) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title(Span::styled(
                source_file.file_name(),
                Style::default().bold(),
            ))
            .title_alignment(Alignment::Center);
        let inner_area = block.inner(area);

        block.render(area, buffer);

        let columns = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [source_area, syntax_area] = columns.areas(inner_area);

        let paragraph = Paragraph::new(source_file.content_as_str())
            .wrap(Wrap { trim: false })
            .scroll((0, 0));

        paragraph.render(source_area, buffer);

        let paragraph = Paragraph::new(syntax_tree.to_string())
            .wrap(Wrap { trim: false })
            .scroll((0, 0));

        paragraph.render(syntax_area, buffer);
    }

    fn draw_prototype_tab(&self, tab: &PrototypeTab, area: Rect, buffer: &mut Buffer) {
        let PrototypeTab { id, prototype } = tab;

        fn get_section_length(line_count: usize) -> u16 {
            if line_count == 0 {
                0
            } else {
                line_count as u16 + 3
            }
        }

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);
        let inner_area = block.inner(area);

        block.render(area, buffer);

        let areas = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(get_section_length(prototype.instructions.len())),
        ]);
        let [id_area, info_area, instructions_area] = areas.flex(Flex::Start).areas(inner_area);

        Paragraph::new(id.to_string())
            .centered()
            .wrap(Wrap { trim: true })
            .render(id_area, buffer);

        Paragraph::new(format!(
            "{} instructions, {} registers",
            prototype.instructions.len(),
            prototype.register_count
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(info_area, buffer);

        let instruction_rows = prototype
            .instructions
            .iter()
            .enumerate()
            .map(|(index, instruction)| {
                [
                    index.to_string(),
                    instruction.operation().to_string(),
                    instruction.disassembly_info(),
                ]
            })
            .collect::<Vec<_>>();
        let instruction_section = BlockTable::new(
            "Instructions",
            ["IP", "Operation", "Info"],
            instruction_rows,
            self.selection_state.current_row,
        );

        instruction_section.render(instructions_area, buffer);
    }

    fn draw_constant_tab(&self, area: Rect, buffer: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);
        let inner_area = block.inner(area);

        block.render(area, buffer);

        let constant_rows = self
            .constant_tags
            .iter()
            .enumerate()
            .map(|(index, tag)| {
                let value = match *tag {
                    OperandType::CHARACTER => self
                        .program
                        .constants
                        .get_character(index as u16)
                        .unwrap()
                        .to_string(),
                    OperandType::I_32 => self
                        .program
                        .constants
                        .get_i32(index as u16)
                        .unwrap()
                        .to_string(),
                    OperandType::I_64 => self
                        .program
                        .constants
                        .get_i64(index as u16)
                        .unwrap()
                        .to_string(),
                    OperandType::I_128 => self
                        .program
                        .constants
                        .get_i128(index as u16)
                        .unwrap()
                        .to_string(),
                    OperandType::U_32 => self
                        .program
                        .constants
                        .get_u32(index as u16)
                        .unwrap()
                        .to_string(),
                    OperandType::U_64 => self
                        .program
                        .constants
                        .get_u64(index as u16)
                        .unwrap()
                        .to_string(),
                    OperandType::U_128 => self
                        .program
                        .constants
                        .get_u128(index as u16)
                        .unwrap()
                        .to_string(),
                    OperandType::F_32 => self
                        .program
                        .constants
                        .get_f32(index as u16)
                        .unwrap()
                        .to_string(),
                    OperandType::F_64 => self
                        .program
                        .constants
                        .get_f64(index as u16)
                        .unwrap()
                        .to_string(),
                    _ => "<error>".to_string(),
                };

                [tag.to_string(), value]
            })
            .collect::<Vec<_>>();
        let constant_section = BlockTable::new(
            "Constants",
            ["Type", "Value"],
            constant_rows,
            self.selection_state.current_row,
        );

        constant_section.render(inner_area, buffer);
    }
}

impl Widget for &mut Disassembler<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer)
    where
        Self: Sized,
    {
        let frame_areas = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .margin(1);
        let [
            title_area,
            _,
            program_name_area,
            program_info_area,
            _,
            tab_header_area,
            tab_content_area,
        ] = frame_areas.areas(area);

        Paragraph::new("Dust Disassembler".bold())
            .centered()
            .wrap(Wrap { trim: true })
            .render(title_area, buffer);

        Paragraph::new(self.program.name().as_str())
            .centered()
            .wrap(Wrap { trim: true })
            .render(program_name_area, buffer);

        Paragraph::new(format!(
            "main function returns {}",
            self.program.return_type()
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(program_info_area, buffer);

        Tabs::new(&self.tabs)
            .highlight_style(Style::default().cyan().bold())
            .select(self.selection_state.current_tab)
            .render(tab_header_area, buffer);

        match &self.tabs[self.selection_state.current_tab] {
            Tab::SourceFile {
                file_name: _,
                file_id,
            } => {
                let source_file = self.source.get_code(*file_id).unwrap();
                let syntax_tree = self.syntax.get_tree(*file_id).unwrap();

                self.draw_source_tab(source_file, syntax_tree, tab_content_area, buffer);
            }
            Tab::Constants => self.draw_constant_tab(tab_content_area, buffer),
            Tab::Prototype(tab) => {
                self.selection_state.row_count = tab.prototype.instructions.len();

                self.draw_prototype_tab(tab, tab_content_area, buffer);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TuiState {
    Run,
    Quit,
}

struct SelectionState {
    tab_count: usize,
    current_tab: usize,

    row_count: usize,
    current_row: Option<usize>,
}

impl SelectionState {
    fn next_tab(&mut self) {
        if self.current_tab < self.tab_count - 1 {
            self.current_tab += 1;
        } else {
            self.current_tab = 0;
        }

        self.current_row = None;
    }

    fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = self.tab_count - 1;
        }

        self.current_row = None;
    }

    fn next_row(&mut self) {
        if let Some(current_row) = self.current_row {
            let last_row = self.row_count.saturating_sub(1);

            if current_row == last_row {
                self.current_row = None;
            } else {
                let next_row = (current_row + 1).min(last_row);

                self.current_row = Some(next_row);
            }
        } else {
            self.current_row = Some(0);
        }
    }

    fn previous_row(&mut self) {
        if let Some(current_row) = self.current_row {
            if current_row > 0 {
                self.current_row = Some(current_row.saturating_sub(1));
            } else {
                self.current_row = None;
            }
        } else {
            self.current_row = Some(self.row_count.saturating_sub(1));
        }
    }
}

enum Tab<'a> {
    SourceFile { file_name: &'a str, file_id: FileId },
    Constants,
    Prototype(PrototypeTab<'a>),
}

impl<'a> From<&'a Tab<'a>> for Line<'a> {
    fn from(tab: &'a Tab) -> Self {
        match tab {
            Tab::SourceFile { file_name, .. } => {
                let title = Span::raw("Source: ");
                let file_name = Span::styled(*file_name, Style::default().bold());

                Line::from(vec![title, file_name])
            }
            Tab::Constants => Line::from("Constants"),
            Tab::Prototype(PrototypeTab { id, .. }) => Line::from(format!("{id}")),
        }
    }
}

struct PrototypeTab<'a> {
    id: u16,
    prototype: &'a Prototype,
}
