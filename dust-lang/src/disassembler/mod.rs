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
    prelude::PrototypeId,
    program::Program,
    prototype::Prototype,
    resolver::Resolver,
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, SyntaxTree},
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

        for (prototype_id, prototype) in program.prototypes.iter() {
            let found_declaration = resolver
                .declarations
                .find_prototype_declaration(prototype_id)
                .unwrap();
            let function_name = if let Some(declaration) = found_declaration {
                let symbol = resolver.symbols.get_symbol(&declaration.symbol_id).unwrap();

                Some(symbol)
            } else {
                None
            };
            let source = if let Some(declaration) = found_declaration
                && let Some((position, _)) = declaration.syntax
            {
                let content = source.get_file_content(&position).unwrap();

                Some(content)
            } else {
                None
            };

            tabs.push(Tab::Prototype {
                function_name,
                prototype_id,
                source,
            });
        }

        Self {
            program,
            source,
            syntax,
            resolver,
            constant_tags,
            tabs,
            state: TuiState::Run,
            selection_state: SelectionState {
                tab: source.file_count(),
                section: None,
                row: 0,
            },
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
                    if self.selection_state.tab < self.tabs.len() - 1 {
                        self.selection_state.tab += 1;
                    } else {
                        self.selection_state.tab = 0;
                    }

                    self.selection_state.section = None;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if self.selection_state.tab > 0 {
                        self.selection_state.tab -= 1;
                    } else {
                        self.selection_state.tab = self.tabs.len() - 1;
                    }

                    self.selection_state.section = None;
                }
                KeyCode::Up | KeyCode::Char('k')
                    if self.selection_state.tab >= self.syntax.len() =>
                {
                    let prototype_index = self.selection_state.tab - self.syntax.len();
                    let prototype = &self.program.prototypes[prototype_index];
                    if self.selection_state.row > 0 {
                        self.selection_state.row -= 1;
                    } else {
                        match self.selection_state.section {
                            Some(section) => {
                                self.selection_state.section = Some(section.previous());
                                let section_length = match self.selection_state.section {
                                    Some(PrototypeSection::Instructions) => {
                                        prototype.instructions.len()
                                    }
                                    Some(PrototypeSection::Constants) => self.constant_tags.len(),
                                    Some(PrototypeSection::Drops) => prototype.drops.len(),
                                    None => 0,
                                };
                                if section_length > 0 {
                                    self.selection_state.row = section_length - 1;
                                } else {
                                    self.selection_state.row = 0;
                                }
                            }
                            None => {
                                self.selection_state.section = Some(PrototypeSection::Drops);
                                let section_length = prototype.drops.len();
                                if section_length > 0 {
                                    self.selection_state.row = section_length - 1;
                                } else {
                                    self.selection_state.row = 0;
                                }
                            }
                        }
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {}
                KeyCode::Down | KeyCode::Char('j')
                    if self.selection_state.tab >= self.syntax.len() =>
                {
                    let prototype_index = self.selection_state.tab - self.syntax.len();
                    let prototype = &self.program.prototypes[prototype_index];
                    let section_length = match self.selection_state.section {
                        Some(PrototypeSection::Instructions) => prototype.instructions.len(),
                        Some(PrototypeSection::Constants) => self.constant_tags.len(),
                        Some(PrototypeSection::Drops) => prototype.drops.len(),
                        None => 0,
                    };

                    if self.selection_state.row + 1 < section_length {
                        self.selection_state.row += 1;
                    } else {
                        match self.selection_state.section {
                            Some(section) => {
                                self.selection_state.section = Some(section.next());
                                self.selection_state.row = 0;
                            }
                            None => {
                                self.selection_state.section = Some(PrototypeSection::Instructions);
                                self.selection_state.row = 0;
                            }
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {}
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

    fn draw_prototype_tab(
        &self,
        index: usize,
        prototype: &Prototype,
        area: Rect,
        buffer: &mut Buffer,
    ) {
        fn get_section_length(count: usize) -> u16 {
            if count == 0 { 0 } else { count as u16 + 3 }
        }

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);
        let inner_area = block.inner(area);

        block.render(area, buffer);

        let areas = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(get_section_length(prototype.instructions.len())),
            Constraint::Length(get_section_length(prototype.drops.len())),
        ]);
        let [
            prototype_area,
            info_area,
            type_area,
            instructions_area,
            drop_lists_area,
        ] = areas.flex(Flex::Start).areas(inner_area);

        Paragraph::new(format!("proto_{}", index))
            .centered()
            .wrap(Wrap { trim: true })
            .bold()
            .render(prototype_area, buffer);

        Paragraph::new(format!(
            "{} instructions, {} registers",
            prototype.instructions.len(),
            prototype.register_count
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(info_area, buffer);

        Paragraph::new(format!("Return type: {}", prototype.return_type))
            .centered()
            .wrap(Wrap { trim: true })
            .render(type_area, buffer);

        // Instructions section
        {
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
            let selected_row =
                if self.selection_state.section == Some(PrototypeSection::Instructions) {
                    Some(self.selection_state.row)
                } else {
                    None
                };
            let instruction_section = BlockTable::new(
                "Instructions",
                ["IP", "Operation", "Info"],
                instruction_rows,
                selected_row,
            );

            instruction_section.render(instructions_area, buffer);
        }

        // Drops section
        if !prototype.drops.is_empty() {
            let drop_list_rows = prototype
                .drops
                .iter()
                .enumerate()
                .map(|(index, register)| [index.to_string(), format!("reg_{register}")])
                .collect::<Vec<_>>();
            let selected_row = if self.selection_state.section == Some(PrototypeSection::Drops) {
                Some(self.selection_state.row)
            } else {
                None
            };
            let drop_lists_section = BlockTable::new(
                "Drop List",
                ["i", "Drop List"],
                drop_list_rows,
                selected_row,
            );

            drop_lists_section.render(drop_lists_area, buffer);
        }
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
            prototype_tabs_header_area,
            tab_content_area,
        ] = frame_areas.areas(area);

        Paragraph::new("Dust Disassembler".bold())
            .centered()
            .wrap(Wrap { trim: true })
            .render(title_area, buffer);

        let main_prototype = &self.program.prototypes[0];
        let program_name = self.program.name();

        Paragraph::new(format!("program: {program_name}",))
            .centered()
            .wrap(Wrap { trim: true })
            .render(program_name_area, buffer);

        Paragraph::new(format!(
            "main function type: {} ({} other prototypes)",
            main_prototype.return_type,
            self.program.prototypes.len() - 1,
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(program_info_area, buffer);

        Tabs::new(&self.tabs)
            .highlight_style(Style::default().cyan().bold())
            .select(self.selection_state.tab)
            .render(prototype_tabs_header_area, buffer);

        if self.selection_state.tab == self.source.file_count() + self.program.prototypes.len() {
            self.draw_declaration_tab(self.resolver, tab_content_area, buffer);
        } else if self.selection_state.tab < self.source.file_count() {
            let source_file = self.source.files().get(self.selection_state.tab).unwrap();
            let syntax_tree = self.syntax.iter().nth(self.selection_state.tab).unwrap();

            self.draw_source_tab(source_file, syntax_tree, tab_content_area, buffer);
        } else {
            let prototype_index = self.selection_state.tab - self.source.file_count();
            let prototype = &self.program.prototypes[prototype_index];

            self.draw_prototype_tab(prototype_index, prototype, tab_content_area, buffer);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TuiState {
    Run,
    Quit,
}

struct SelectionState {
    tab: usize,
    section: Option<PrototypeSection>,
    row: usize,
}

enum Tab<'src> {
    SourceFile {
        file_name: &'src str,
        file_id: SourceFileId,
    },
    Declarations,
    Prototype {
        function_name: Option<&'src str>,
        prototype_id: PrototypeId,
        source: Option<&'src str>,
    },
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
            Tab::Prototype { function_name, .. } => {
                let title = Span::raw("Prototype: ");
                let function_name = if let Some(function_name) = function_name {
                    Span::styled(*function_name, Style::default().bold())
                } else {
                    Span::styled("anonymous", Style::default().bold())
                };

                Line::from(vec![title, function_name])
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PrototypeSection {
    Instructions,
    Constants,
    Drops,
}

impl PrototypeSection {
    fn next(&self) -> PrototypeSection {
        match self {
            PrototypeSection::Instructions => PrototypeSection::Constants,
            PrototypeSection::Constants => PrototypeSection::Drops,
            PrototypeSection::Drops => PrototypeSection::Instructions,
        }
    }

    fn previous(&self) -> PrototypeSection {
        match self {
            PrototypeSection::Instructions => PrototypeSection::Drops,
            PrototypeSection::Constants => PrototypeSection::Instructions,
            PrototypeSection::Drops => PrototypeSection::Constants,
        }
    }
}
