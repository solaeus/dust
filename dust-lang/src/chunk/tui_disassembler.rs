use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, Tabs, Widget, Wrap},
};

use crate::{Chunk, Source, Type, dust_crate::Program};

pub struct TuiDisassembler<'a> {
    program: &'a Program,
    source: Source,
    state: TuiState,
    selected_tab: usize,
    tabs: Vec<String>,
}

impl<'a> TuiDisassembler<'a> {
    pub fn new(program: &'a Program, source: Source) -> Self {
        let mut tabs = Vec::with_capacity(source.len() + program.prototypes.len());

        for file in source.get_files() {
            tabs.push(file.name.to_string());
        }

        for chunk in &program.prototypes {
            tabs.push(
                chunk
                    .get_name(&program.resolver, &source)
                    .expect("Failed to get chunk name")
                    .to_string(),
            );
        }

        Self {
            program,
            source,
            state: TuiState::Run,
            selected_tab: 0,
            tabs,
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
        source_code: String,
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

        let paragraph = Paragraph::new(source_code)
            .wrap(Wrap { trim: false })
            .scroll((0, 0));

        paragraph.render(inner_area, buffer);
    }

    fn draw_chunk_tab(&self, index: usize, chunk: &Chunk, area: Rect, buffer: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);
        let inner_area = block.inner(area);

        block.render(area, buffer);

        let areas = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(chunk.instructions.len() as u16 + 3),
            Constraint::Length(1),
            Constraint::Length(self.program.constants.len() as u16 + 3),
            Constraint::Length(1),
            Constraint::Length(chunk.call_arguments.len() as u16 + 3),
            Constraint::Fill(1),
        ]);
        let [
            prototype_area,
            _,
            info_area,
            _,
            type_area,
            _,
            instructions_area,
            _,
            constants_area,
            _,
            arguments_area,
            _,
        ] = areas.areas(inner_area);

        Paragraph::new(format!("proto_{}", index))
            .centered()
            .wrap(Wrap { trim: true })
            .render(prototype_area, buffer);

        Paragraph::new(format!(
            "{} instructions, {} constants",
            chunk.instructions.len(),
            self.program.constants.len(),
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(info_area, buffer);

        let chunk_type = self
            .program
            .resolver
            .resolve_type(chunk.type_id)
            .unwrap_or(Type::None);

        Paragraph::new(chunk_type.to_string())
            .centered()
            .wrap(Wrap { trim: true })
            .render(type_area, buffer);

        // Instructions section
        {
            let horizontal_areas =
                Layout::horizontal([Constraint::Min(1), Constraint::Min(60), Constraint::Min(1)]);
            let [_, instructions_block_area, _] = horizontal_areas.areas(instructions_area);

            let instructions_block = Block::new()
                .title(Span::styled("Instructions", Style::default()))
                .title_alignment(Alignment::Center)
                .title_style(Style::default().bold())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);
            let instructions_table_area = instructions_block.inner(instructions_block_area);

            instructions_block.render(instructions_block_area, buffer);

            let horizontal_areas =
                Layout::horizontal([Constraint::Min(1), Constraint::Min(50), Constraint::Min(1)]);
            let [_, center_area, _] = horizontal_areas.areas(instructions_table_area);

            let instruction_rows = chunk
                .instructions
                .iter()
                .enumerate()
                .map(|(ip, instruction)| {
                    Row::new(vec![
                        ip.to_string(),
                        instruction.operation().to_string(),
                        instruction.disassembly_info(),
                    ])
                })
                .collect::<Vec<_>>();
            let widths = [
                Constraint::Length(5),
                Constraint::Length(12),
                Constraint::Length(30),
            ];

            let instructions_table = Table::new(instruction_rows, widths)
                .header(Row::new(["IP", "Operation", "Info"]).add_modifier(Modifier::BOLD));

            instructions_table.render(center_area, buffer);
        }

        // Constants section
        {
            let horizontal_areas =
                Layout::horizontal([Constraint::Min(1), Constraint::Min(60), Constraint::Min(1)]);
            let [_, block_area, _] = horizontal_areas.areas(constants_area);

            let block = Block::new()
                .title(Span::styled("Constants", Style::default()))
                .title_alignment(Alignment::Center)
                .title_style(Style::default().bold())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);

            let instructions_table_area = block.inner(block_area);

            block.render(block_area, buffer);

            let horizontal_areas =
                Layout::horizontal([Constraint::Min(1), Constraint::Min(40), Constraint::Min(1)]);
            let [_, center_area, _] = horizontal_areas.areas(instructions_table_area);

            let instruction_rows = self
                .program
                .constants
                .iter()
                .enumerate()
                .map(|(index, value)| Row::new(vec![format!("const_{index}"), value.to_string()]))
                .collect::<Vec<_>>();
            let widths = [Constraint::Min(30), Constraint::Min(30)];

            let instructions_table = Table::new(instruction_rows, widths)
                .header(Row::new(["Address", "Value"]).add_modifier(Modifier::BOLD));

            instructions_table.render(center_area, buffer);
        }

        // Arguments section
        {
            let horizontal_areas =
                Layout::horizontal([Constraint::Min(1), Constraint::Min(60), Constraint::Min(1)]);
            let [_, block_area, _] = horizontal_areas.areas(arguments_area);

            let block = Block::new()
                .title(Span::styled("Arguments", Style::default()))
                .title_alignment(Alignment::Center)
                .title_style(Style::default().bold())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);

            let instructions_table_area = block.inner(block_area);

            block.render(block_area, buffer);

            let horizontal_areas =
                Layout::horizontal([Constraint::Min(1), Constraint::Min(40), Constraint::Min(1)]);
            let [_, center_area, _] = horizontal_areas.areas(instructions_table_area);

            let address_rows = chunk
                .call_arguments
                .iter()
                .enumerate()
                .map(|(index, (address, r#type))| {
                    Row::new(vec![
                        index.to_string(),
                        address.to_string(*r#type),
                        r#type.to_string(),
                    ])
                })
                .collect::<Vec<_>>();
            let widths = [
                Constraint::Length(5),
                Constraint::Min(10),
                Constraint::Min(10),
            ];

            let address_table = Table::new(address_rows, widths)
                .header(Row::new(["i", "Address", "Type"]).add_modifier(Modifier::BOLD));

            address_table.render(center_area, buffer);
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

        Paragraph::new(format!("program: {}", self.program.name))
            .centered()
            .wrap(Wrap { trim: true })
            .render(program_name_area, buffer);

        let main_chunk_type = self
            .program
            .resolver
            .resolve_type(main_chunk.type_id)
            .unwrap_or(Type::None);

        Paragraph::new(format!(
            "main function type: {main_chunk_type} ({} other prototypes)",
            self.program.prototypes.len() - 1,
        ))
        .centered()
        .wrap(Wrap { trim: true })
        .render(program_info_area, buffer);

        Tabs::new(self.tabs.clone())
            .highlight_style(Style::default().cyan().bold())
            .select(self.selected_tab)
            .render(chunk_tabs_header_area, buffer);

        if self.selected_tab < self.source.len() {
            let source_file = self.source.get_files().get(self.selected_tab).unwrap();

            self.draw_source_tab(
                &source_file.name,
                source_file.source_code.clone(),
                tab_content_area,
                buffer,
            );
        } else {
            let chunk_index = self.selected_tab - self.source.len();
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
