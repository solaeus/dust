use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Row, ScrollbarState, Table, Widget},
};

const COLUMN_SPACING: u16 = 2;

pub struct BlockTable<const COLUMN_COUNT: usize> {
    title: &'static str,
    headers: [&'static str; COLUMN_COUNT],
    rows: Vec<[String; COLUMN_COUNT]>,
    scrollbar_state: ScrollbarState,
}

impl<const COLUMN_COUNT: usize> BlockTable<COLUMN_COUNT> {
    pub fn new(
        title: &'static str,
        headers: [&'static str; COLUMN_COUNT],
        rows: Vec<[String; COLUMN_COUNT]>,
    ) -> Self {
        Self {
            title,
            headers,
            scrollbar_state: ScrollbarState::new(rows.len()),
            rows,
        }
    }
}

impl<const COLUMN_COUNT: usize> Widget for BlockTable<COLUMN_COUNT>
where
    [&'static str; COLUMN_COUNT * 2 + 1]: Sized,
{
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let mut column_widths = [0; COLUMN_COUNT];

        for (index, header) in self.headers.iter().enumerate() {
            column_widths[index] = header.chars().count() + 2;
        }

        for row in &self.rows {
            for (index, cell) in row.iter().enumerate() {
                column_widths[index] = column_widths[index].max(cell.chars().count() + 2);
            }
        }

        let table_width =
            column_widths.iter().sum::<usize>() as u16 + (COLUMN_SPACING * COLUMN_COUNT as u16) + 2;
        let [_, table_area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Min(table_width),
            Constraint::Fill(1),
        ])
        .areas(area);
        let block = Block::new()
            .title(Span::styled(self.title, Style::default()))
            .title_alignment(Alignment::Center)
            .title_style(Style::default().bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let rows = self.rows.into_iter().map(Row::new);
        let columns = column_widths
            .into_iter()
            .map(|width| Constraint::Length(width as u16));
        let table = Table::new(rows, columns)
            .header(Row::new(self.headers).add_modifier(Modifier::BOLD))
            .column_spacing(COLUMN_SPACING)
            .flex(Flex::SpaceAround)
            .block(block);

        table.render(table_area, buffer);
    }
}
