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

use crate::{
    chunk::Chunk, dust_crate::Program, resolver::Resolver, source::Source, syntax_tree::SyntaxTree,
};

use block_table::BlockTable;

pub struct TuiDisassembler<'a> {
    program: &'a Program,
    source: &'a Source,
    file_trees: &'a [SyntaxTree],
    _resolver: &'a Resolver,

    state: TuiState,
    selected_tab: usize,
    tabs: Vec<String>,
}

impl<'a> TuiDisassembler<'a> {
    pub fn new(
        program: &'a Program,
        source: &'a Source,
        file_trees: &'a [SyntaxTree],
        resolver: &'a Resolver,
    ) -> Self {
        let files = source.read_files();
        let mut tabs = Vec::with_capacity(files.len() + program.prototypes.len());

        for file in files.iter() {
            tabs.push(file.name.clone());
        }

        for (index, chunk) in program.prototypes.iter().enumerate() {
            let chunk_name = if index == 0 {
                "main".to_string()
            } else if let Some(name_position) = chunk.name_position {
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
            tabs.push(chunk_name);
        }

        Self {
            selected_tab: files.len(),
            program,
            source,
            file_trees,
            _resolver: resolver,
            tabs,
            state: TuiState::Run,
        }
    }

    pub fn disassemble(mut self) -> io::Result<()> {
        let mut terminal = ratatui::init();

        while self.state == TuiState::Run {
            if let Err(error) = terminal.draw(|frame| frame.render_widget(&self, frame.area())) {
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
                KeyCode::Char('l') | KeyCode::Right => {
                    if self.selected_tab < self.tabs.len() - 1 {
                        self.selected_tab += 1;
                    } else {
                        self.selected_tab = 0;
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if self.selected_tab > 0 {
                        self.selected_tab -= 1;
                    } else {
                        self.selected_tab = self.tabs.len() - 1;
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => {
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

    fn draw_chunk_tab(&self, index: usize, chunk: &Chunk, area: Rect, buffer: &mut Buffer) {
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
            Constraint::Length(get_section_length(chunk.instructions.len())),
            Constraint::Length(get_section_length(self.program.constants.len())),
            Constraint::Length(get_section_length(chunk.call_arguments.len())),
            Constraint::Length(get_section_length(chunk.drop_lists.len())),
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
            chunk.instructions.len(),
            chunk.register_count,
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(info_area, buffer);

        Paragraph::new(format!("Function type: {}", chunk.function_type))
            .centered()
            .wrap(Wrap { trim: true })
            .render(type_area, buffer);

        // Instructions section
        {
            let instruction_rows = chunk
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
            );

            instruction_section.render(instructions_area, buffer);
        }

        // Constants section
        if !self.program.constants.is_empty() {
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
            let constants_section =
                BlockTable::new("Constants", ["Address", "Value", "Type"], constant_rows);

            constants_section.render(constants_area, buffer);
        }

        // Arguments section
        if !chunk.call_arguments.is_empty() {
            let argument_rows = chunk
                .call_arguments
                .iter()
                .enumerate()
                .map(|(index, (address, operand_type))| {
                    [
                        index.to_string(),
                        format!("reg_{}", address.to_string(*operand_type)),
                        operand_type.to_string(),
                    ]
                })
                .collect::<Vec<_>>();
            let arguments_section =
                BlockTable::new("Call Arguments", ["i", "Address", "Type"], argument_rows);

            arguments_section.render(arguments_area, buffer);
        }

        // Drops section
        if !chunk.drop_lists.is_empty() {
            let drop_list_rows = chunk
                .drop_lists
                .iter()
                .enumerate()
                .map(|(index, register)| [index.to_string(), format!("reg_{register}")])
                .collect::<Vec<_>>();
            let drop_lists_section =
                BlockTable::new("Drop List", ["i", "Drop List"], drop_list_rows);

            drop_lists_section.render(drop_lists_area, buffer);
        }
    }
}

impl Widget for &TuiDisassembler<'_> {
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
            chunk_tabs_header_area,
            tab_content_area,
        ] = frame_areas.areas(area);

        Paragraph::new("Dust Disassembler".bold())
            .centered()
            .wrap(Wrap { trim: true })
            .render(title_area, buffer);

        let main_chunk = &self.program.main_chunk();
        let files = self.source.read_files();
        let main_chunk_name = main_chunk
            .name_position
            .and_then(|pos| files.get(pos.file_id.0 as usize))
            .map(|file| file.name.as_str())
            .unwrap_or("unknown");

        Paragraph::new(format!("program: {main_chunk_name}",))
            .centered()
            .wrap(Wrap { trim: true })
            .render(program_name_area, buffer);

        Paragraph::new(format!(
            "main function type: {} ({} other prototypes)",
            main_chunk.function_type,
            self.program.prototypes.len() - 1,
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(program_info_area, buffer);

        Tabs::new(self.tabs.clone())
            .highlight_style(Style::default().cyan().bold())
            .select(self.selected_tab)
            .render(chunk_tabs_header_area, buffer);

        if self.selected_tab < files.len() {
            let source_file = files.get(self.selected_tab).unwrap();

            self.draw_source_tab(
                &source_file.name,
                unsafe { str::from_utf8_unchecked(source_file.source_code.as_ref()) },
                &self.file_trees[self.selected_tab],
                tab_content_area,
                buffer,
            );
        } else {
            let chunk_index = self.selected_tab - files.len();
            let chunk = &self.program.prototypes[chunk_index];

            self.draw_chunk_tab(chunk_index, chunk, tab_content_area, buffer);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TuiState {
    Run,
    Quit,
}
