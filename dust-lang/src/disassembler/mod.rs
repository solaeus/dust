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
use tracing::error;

use crate::{
    instruction::OperandType,
    program::Program,
    prototype::{Prototype, PrototypeId},
    resolver::Resolver,
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, tree::SyntaxTree},
};

use block_table::BlockTable;

pub struct Disassembler<'a> {
    program: &'a Program,
    source: &'a Source<'a>,
    syntax: &'a Syntax,
    resolver: &'a Resolver,
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
        resolver: &'a Resolver,
        constant_tags: &'a [OperandType],
    ) -> Self {
        let mut tabs = Vec::with_capacity(source.file_count() + program.prototypes.len() + 1);

        for (file_id, file) in source.iter() {
            tabs.push(Tab::SourceFile {
                file_name: file.file_name(),
                file_id,
            });
        }

        tabs.push(Tab::Declarations);

        for (id, prototype) in program.prototypes.iter() {
            let name_display = if let Some(symbol_id) = prototype.debug_symbol_id {
                let symbol = resolver.symbols.get_symbol(&symbol_id).unwrap();

                symbol.to_string()
            } else {
                let file = source.get_file(prototype.debug_position.file_id).unwrap();
                let file_name = file.file_name();
                let span_range = prototype.debug_position.span.as_usize_range();
                let (start_line, start_column) = file.content_as_str()[..span_range.start]
                    .lines()
                    .fold((1, 0), |(line, column), line_content| {
                        let line_length = line_content.chars().count() + 1;

                        if column + line_length > span_range.start {
                            (line, column)
                        } else {
                            (line + 1, column + line_length)
                        }
                    });
                let (end_line, end_column) = file.content_as_str()
                    [span_range.start..span_range.end]
                    .lines()
                    .fold((start_line, 0), |(line, column), line_content| {
                        let line_length = line_content.chars().count() + 1;

                        if column + line_length > span_range.end {
                            (line, column)
                        } else {
                            (line + 1, column + line_length)
                        }
                    });

                format!(
                    "closure @ {file_name} {start_line}:{start_column}..{end_line}:{end_column}"
                )
            };

            let (source, source_lines, source_width) = {
                let content = source.get_file_content(&prototype.debug_position).unwrap();
                let mut line_count = 0;
                let mut max_width = 0;

                for line in content.lines() {
                    line_count += 1;
                    max_width = max_width.max(line.chars().count() as u16);
                }

                (Some(content), line_count, max_width)
            };

            tabs.push(Tab::Prototype(PrototypeTab {
                id,
                prototype,
                name_display,
                source,
                source_lines,
                source_width,
            }));
        }

        Self {
            program,
            source,
            syntax,
            resolver,
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
        source_file: &SourceFile,
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
        let PrototypeTab {
            id,
            prototype,
            name_display,
            source,
            source_lines,
            source_width,
        } = tab;

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
            Constraint::Length(source_lines + 1),
            Constraint::Length(2),
            Constraint::Length(get_section_length(prototype.instructions.len())),
        ]);
        let [
            name_area,
            id_area,
            source_area,
            info_area,
            instructions_area,
            _drop_lists_area,
        ] = areas.flex(Flex::Start).areas(inner_area);

        Paragraph::new(name_display.as_str())
            .centered()
            .wrap(Wrap { trim: true })
            .bold()
            .render(name_area, buffer);

        Paragraph::new(id.to_string())
            .centered()
            .wrap(Wrap { trim: true })
            .render(id_area, buffer);

        if let Some(source) = source {
            let areas = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(*source_width),
                Constraint::Fill(1),
            ])
            .areas(source_area);
            let [_, source_area, _] = areas;

            Paragraph::new(*source)
                .wrap(Wrap { trim: false })
                .render(source_area, buffer);
        }

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

    fn draw_declaration_tab(&self, resolver: &'a Resolver, area: Rect, buffer: &mut Buffer) {
        let declaration_displays = resolver
            .declaration_display_iterator()
            .map(|result| match result {
                Ok(display) => [display],
                Err(error) => {
                    error!("{error:?}");

                    ["Error".to_string()]
                }
            })
            .collect();

        BlockTable::new("Declarations", [""], declaration_displays, None).render(area, buffer);
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
                let source_file = self.source.get_file(*file_id).unwrap();
                let syntax_tree = self.syntax.get_tree(*file_id).unwrap();

                self.draw_source_tab(source_file, syntax_tree, tab_content_area, buffer);
            }
            Tab::Declarations => self.draw_declaration_tab(self.resolver, tab_content_area, buffer),
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
    SourceFile {
        file_name: &'a str,
        file_id: SourceFileId,
    },
    Declarations,
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
            Tab::Declarations => Line::from("Declarations"),
            Tab::Prototype(PrototypeTab { name_display, .. }) => {
                let title = Span::raw("Prototype: ");
                let name_display = Span::raw(name_display.as_str());

                Line::from(vec![title, name_display])
            }
        }
    }
}

struct PrototypeTab<'a> {
    id: PrototypeId,
    prototype: &'a Prototype,
    name_display: String,
    source: Option<&'a str>,
    source_lines: u16,
    source_width: u16,
}
