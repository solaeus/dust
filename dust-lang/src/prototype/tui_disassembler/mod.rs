mod block_table;

use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph, Tabs, Widget, Wrap},
};

use crate::{dust_crate::Program, prototype::Prototype, source::Source, syntax_tree::SyntaxTree};

use block_table::BlockTable;

pub struct TuiDisassembler<'a> {
    program: &'a Program,
    source: &'a Source,
    file_trees: &'a [SyntaxTree],

    show_constants: bool,
    show_arguments: bool,
    show_drops: bool,

    state: TuiState,
    selection_state: SelectionState,
    tabs: Vec<String>,
}

impl<'a> TuiDisassembler<'a> {
    pub fn new(program: &'a Program, source: &'a Source, file_trees: &'a [SyntaxTree]) -> Self {
        let files = source.read_files();
        let mut tabs = Vec::with_capacity(files.len() + program.prototypes.len());

        for file in files.iter() {
            tabs.push(file.name.clone());
        }

        for (index, prototype) in program.prototypes.iter().enumerate() {
            let prototype_name = if index == 0 {
                "main".to_string()
            } else if let Some(name_position) = prototype.name_position {
                files
                    .get(name_position.file_id.0 as usize)
                    .and_then(|file| {
                        file.source_code
                            .as_ref()
                            .get(name_position.span.as_usize_range())
                            .map(|bytes| unsafe { str::from_utf8_unchecked(bytes) })
                    })
                    .unwrap_or("anonymous")
                    .to_string()
            } else {
                "anonymous".to_string()
            };
            tabs.push(prototype_name);
        }

        Self {
            program,
            source,
            file_trees,

            show_constants: !program.constants.is_empty(),
            show_arguments: false,
            show_drops: false,

            state: TuiState::Run,
            selection_state: SelectionState {
                tab: files.len(),
                section: PrototypeSection::Instructions,
                row: 0,
            },
            tabs,
        }
    }

    pub fn disassemble(mut self) -> io::Result<()> {
        let mut terminal = ratatui::init();

        while self.state == TuiState::Run {
            if let Err(error) = terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))
            {
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

                    self.selection_state.section = PrototypeSection::Instructions;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if self.selection_state.tab > 0 {
                        self.selection_state.tab -= 1;
                    } else {
                        self.selection_state.tab = self.tabs.len() - 1;
                    }

                    self.selection_state.section = PrototypeSection::Instructions;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selection_state.tab >= self.file_trees.len()
                        && self.selection_state.row > 0
                    {
                        self.selection_state.row -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selection_state.tab >= self.file_trees.len() {
                        let prototype_index = self.selection_state.tab - self.file_trees.len();
                        let prototype = &self.program.prototypes[prototype_index];
                        let section_length = match self.selection_state.section {
                            PrototypeSection::Instructions => prototype.instructions.len(),
                            PrototypeSection::Constants => self.program.constants.len(),
                            PrototypeSection::CallArguments => prototype.call_arguments.len(),
                            PrototypeSection::DropLists => prototype.drop_lists.len(),
                        };

                        if self.selection_state.row + 1 < section_length {
                            self.selection_state.row += 1;
                        }
                    }
                }
                KeyCode::PageUp | KeyCode::Char('K') => {
                    if self.selection_state.tab >= self.file_trees.len() {
                        self.selection_state.section = self.selection_state.section.previous();
                    }
                }
                KeyCode::PageDown | KeyCode::Char('J') => {
                    if self.selection_state.tab >= self.file_trees.len() {
                        self.selection_state.section = self.selection_state.section.next();
                    }
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
        file_name: &str,
        source_code: &str,
        syntax_tree: &SyntaxTree,
        area: Rect,
        buffer: &mut Buffer,
    ) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title(Span::styled(file_name, Style::default().bold()))
            .title_alignment(Alignment::Center);
        let inner_area = block.inner(area);

        block.render(area, buffer);

        let columns = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [source_area, syntax_area] = columns.areas(inner_area);

        let paragraph = Paragraph::new(source_code)
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
            Constraint::Length(get_section_length(self.program.constants.len())),
            Constraint::Length(get_section_length(prototype.call_arguments.len())),
            Constraint::Length(get_section_length(prototype.drop_lists.len())),
        ]);
        let [
            prototype_area,
            info_area,
            type_area,
            instructions_area,
            constants_area,
            arguments_area,
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
            prototype.register_count,
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(info_area, buffer);

        Paragraph::new(format!("Function type: {}", prototype.function_type))
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
            let selected_row = if self.selection_state.section == PrototypeSection::Instructions {
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

        // Constants section
        if self.show_constants {
            let constant_rows = self
                .program
                .constants
                .display_iterator()
                .enumerate()
                .map(|(index, (value, r#type))| {
                    [
                        format!("const_{index}"),
                        value.to_string(),
                        r#type.to_string(),
                    ]
                })
                .collect::<Vec<_>>();
            let selected_row = if self.selection_state.section == PrototypeSection::Constants {
                Some(self.selection_state.row)
            } else {
                None
            };
            let constants_section = BlockTable::new(
                "Constants",
                ["Address", "Value", "Type"],
                constant_rows,
                selected_row,
            );

            constants_section.render(constants_area, buffer);
        }

        // Arguments section
        if self.show_arguments {
            let argument_rows = prototype
                .call_arguments
                .iter()
                .enumerate()
                .map(|(index, (address, operand_type))| {
                    [
                        index.to_string(),
                        address.to_string(*operand_type),
                        operand_type.to_string(),
                    ]
                })
                .collect::<Vec<_>>();
            let selected_row = if self.selection_state.section == PrototypeSection::CallArguments {
                Some(self.selection_state.row)
            } else {
                None
            };
            let arguments_section = BlockTable::new(
                "Call Arguments",
                ["i", "Address", "Type"],
                argument_rows,
                selected_row,
            );

            arguments_section.render(arguments_area, buffer);
        }

        // Drops section
        if self.show_drops {
            let drop_list_rows = prototype
                .drop_lists
                .iter()
                .enumerate()
                .map(|(index, register)| [index.to_string(), format!("reg_{register}")])
                .collect::<Vec<_>>();
            let selected_row = if self.selection_state.section == PrototypeSection::DropLists {
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
}

impl Widget for &mut TuiDisassembler<'_> {
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

        let main_prototype = &self.program.main_prototype();
        let files = self.source.read_files();
        let main_prototype_name = main_prototype
            .name_position
            .and_then(|pos| files.get(pos.file_id.0 as usize))
            .map(|file| file.name.as_str())
            .unwrap_or("unknown");

        Paragraph::new(format!("program: {main_prototype_name}",))
            .centered()
            .wrap(Wrap { trim: true })
            .render(program_name_area, buffer);

        Paragraph::new(format!(
            "main function type: {} ({} other prototypes)",
            main_prototype.function_type,
            self.program.prototypes.len() - 1,
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(program_info_area, buffer);

        Tabs::new(self.tabs.clone())
            .highlight_style(Style::default().cyan().bold())
            .select(self.selection_state.tab)
            .render(prototype_tabs_header_area, buffer);

        if self.selection_state.tab < files.len() {
            let source_file = files.get(self.selection_state.tab).unwrap();

            self.draw_source_tab(
                &source_file.name,
                unsafe { str::from_utf8_unchecked(source_file.source_code.as_ref()) },
                &self.file_trees[self.selection_state.tab],
                tab_content_area,
                buffer,
            );
        } else {
            let prototype_index = self.selection_state.tab - files.len();
            let prototype = &self.program.prototypes[prototype_index];

            self.show_arguments = !prototype.call_arguments.is_empty();
            self.show_drops = !prototype.drop_lists.is_empty();

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
    section: PrototypeSection,
    row: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PrototypeSection {
    Instructions,
    Constants,
    CallArguments,
    DropLists,
}

impl PrototypeSection {
    fn next(&self) -> PrototypeSection {
        match self {
            PrototypeSection::Instructions => PrototypeSection::Constants,
            PrototypeSection::Constants => PrototypeSection::CallArguments,
            PrototypeSection::CallArguments => PrototypeSection::DropLists,
            PrototypeSection::DropLists => PrototypeSection::Instructions,
        }
    }

    fn previous(&self) -> PrototypeSection {
        match self {
            PrototypeSection::Instructions => PrototypeSection::DropLists,
            PrototypeSection::Constants => PrototypeSection::Instructions,
            PrototypeSection::CallArguments => PrototypeSection::Constants,
            PrototypeSection::DropLists => PrototypeSection::CallArguments,
        }
    }
}
