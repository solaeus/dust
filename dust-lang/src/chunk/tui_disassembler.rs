// use std::io;

// use ratatui::{
//     buffer::Buffer,
//     crossterm::event::{self, Event, KeyCode, KeyEventKind},
//     layout::{Alignment, Constraint, Layout, Rect},
//     style::{Modifier, Style, Stylize},
//     text::Span,
//     widgets::{Block, Borders, Paragraph, Row, Table, Tabs, Widget, Wrap, block::Position},
// };

// use crate::{Chunk, Program};

// pub struct TuiDisassembler<'a> {
//     program: &'a Program,
//     source: Option<&'a str>,
//     state: TuiState,
//     selected_tab: usize,
//     tab_count: usize,
// }

// impl<'a> TuiDisassembler<'a> {
//     pub fn new(program: &'a Program, source: Option<&'a str>) -> Self {
//         Self {
//             program,
//             source,
//             state: TuiState::Run,
//             selected_tab: 0,
//             tab_count: program.prototypes.len() + 1,
//         }
//     }

//     pub fn disassemble(mut self) -> io::Result<()> {
//         let mut terminal = ratatui::init();

//         while self.state == TuiState::Run {
//             if let Err(error) = terminal.draw(|frame| frame.render_widget(&self, frame.area())) {
//                 ratatui::restore();

//                 return Err(error);
//             }
//             self.handle_events()?;
//         }

//         ratatui::restore();

//         Ok(())
//     }

//     fn handle_events(&mut self) -> std::io::Result<()> {
//         if let Event::Key(key) = event::read()?
//             && key.kind == KeyEventKind::Press
//         {
//             match key.code {
//                 KeyCode::Char('l') | KeyCode::Right => {
//                     self.selected_tab = (self.selected_tab + 1).min(self.tab_count - 1)
//                 }
//                 KeyCode::Char('h') | KeyCode::Left => {
//                     self.selected_tab = self.selected_tab.saturating_sub(1);
//                 }
//                 KeyCode::Char('q') | KeyCode::Esc => {
//                     self.state = TuiState::Quit;
//                 }
//                 _ => {}
//             }
//         }

//         Ok(())
//     }

//     fn draw_chunk_tab(&self, chunk: &Chunk, area: Rect, buffer: &mut Buffer) {
//         let function_name = chunk
//             .name
//             .as_ref()
//             .map(|path| path.to_string())
//             .unwrap_or_else(|| "anonymous".to_string());

//         let block = Block::new()
//             .title(Span::styled(
//                 function_name,
//                 Style::default().add_modifier(Modifier::BOLD),
//             ))
//             .title_position(Position::Top)
//             .title_alignment(Alignment::Center)
//             .borders(Borders::ALL);

//         let inner_area = block.inner(area);
//         let areas = Layout::vertical([
//             Constraint::Length(1),
//             Constraint::Length(1),
//             Constraint::Length(1),
//             Constraint::Min(1),
//         ]);
//         let [prototype_area, info_area, type_area, instruction_area] = areas.areas(inner_area);

//         if chunk.prototype_index == u16::MAX {
//             Paragraph::new("main")
//         } else {
//             Paragraph::new(format!("proto_{}", chunk.prototype_index))
//         }
//         .centered()
//         .wrap(Wrap { trim: true })
//         .render(prototype_area, buffer);

//         // Paragraph::new(format!(
//         //     "{} instructions, {} constants, {} locals",
//         //     chunk.instructions.len(),
//         //     chunk.constants.len(),
//         //     chunk.locals.len(),
//         // ))
//         // .centered()
//         // .wrap(Wrap { trim: true })
//         // .render(info_area, buffer);

//         Paragraph::new(format!("{}", chunk.r#type))
//             .centered()
//             .wrap(Wrap { trim: true })
//             .render(type_area, buffer);

//         let instruction_section = {
//             let instruction_rows = chunk
//                 .instructions
//                 .iter()
//                 .enumerate()
//                 .map(|(ip, instruction)| {
//                     Row::new(vec![
//                         ip.to_string(),
//                         instruction.operation().to_string(),
//                         instruction.disassembly_info(),
//                     ])
//                 })
//                 .collect::<Vec<_>>();
//             let widths = [
//                 Constraint::Length(6),
//                 Constraint::Length(20),
//                 Constraint::Min(10),
//             ];

//             Table::new(instruction_rows, widths).header(Row::new(["IP", "Operation", "Info"]))
//         };

//         instruction_section.render(instruction_area, buffer);
//     }
// }

// impl Widget for &TuiDisassembler<'_> {
//     fn render(self, area: Rect, buf: &mut Buffer)
//     where
//         Self: Sized,
//     {
//         let frame_areas = Layout::vertical([
//             Constraint::Length(1),
//             Constraint::Length(1),
//             Constraint::Length(1),
//             Constraint::Min(0),
//         ])
//         .margin(1);
//         let [
//             program_info_area,
//             source_area,
//             chunk_tabs_header_area,
//             chunks_tabs_content_area,
//         ] = frame_areas.areas(area);

//         let Some(main_chunk) = self.program.prototypes.last() else {
//             Paragraph::new("No main chunk found")
//                 .centered()
//                 .wrap(Wrap { trim: true })
//                 .render(program_info_area, buf);

//             return;
//         };

//         Paragraph::new(format!(
//             "\"{}\" has {} declared functions and type {}",
//             main_chunk
//                 .name
//                 .as_ref()
//                 .map(|path| path.to_string())
//                 .unwrap_or_else(|| "anonymous".to_string()),
//             self.program.prototypes.len() - 1,
//             main_chunk.r#type
//         ))
//         .centered()
//         .wrap(Wrap { trim: true })
//         .render(program_info_area, buf);

//         let source = self
//             .source
//             .unwrap_or("<source unavailable>")
//             .split_whitespace()
//             .intersperse(" ")
//             .collect::<String>();

//         Paragraph::new(source)
//             .centered()
//             .wrap(Wrap { trim: true })
//             .render(source_area, buf);

//         let mut function_names = Vec::with_capacity(self.program.prototypes.len() + 1);

//         function_names.push("main".to_string());

//         for prototype in &self.program.prototypes {
//             if let Some(name) = &prototype.name {
//                 function_names.push(name.to_string());
//             } else {
//                 function_names.push("anonymous".to_string());
//             }
//         }

//         Tabs::new(function_names)
//             .highlight_style(Style::default().cyan().on_black())
//             .select(self.selected_tab)
//             .render(chunk_tabs_header_area, buf);

//         match self.selected_tab {
//             0 => {
//                 self.draw_chunk_tab(main_chunk, chunks_tabs_content_area, buf);
//             }
//             _ => {
//                 if let Some(chunk) = self.program.prototypes.get(self.selected_tab - 1) {
//                     self.draw_chunk_tab(chunk, chunks_tabs_content_area, buf);
//                 }
//             }
//         }
//     }
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// enum TuiState {
//     Run,
//     Quit,
// }
