use ratatui::{self, Frame};
// use ratatui::style::{Style};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{BarChart, Block, Borders, Row, Table};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::style::{Style};

fn render(frame :&mut Frame) {
    use Constraint::{Fill, Length, Min};

    let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
    let [title_area, main_area, status_area] = vertical.areas(frame.area());
    let horizontal = Layout::vertical([Fill(1), Fill(2)]);
    let [top_area, bottom_area] = horizontal.areas(main_area);
    let bar = BarChart::default()
        .block(Block::bordered().title("Hello"))
        .data(&[("foo", 1), ("bar", 2), ("baz", 1), ("foo", 2), ("bar", 4), ("baz", 5), ("foo", 1), ("bar", 2),("foo", 1), ("bar", 2)])
        .bar_gap(4)
        .max(10);

    frame.render_widget(Block::bordered().title("Title Bar"), title_area);
    frame.render_widget(Block::bordered().title("Status Bar"), status_area);
    frame.render_widget(Block::bordered().title("Top"), top_area);
    frame.render_widget(Block::bordered().title("Bottom"), bottom_area);
    let bar_layout = Layout::vertical([Fill(1)])
        .horizontal_margin(2)
        .vertical_margin(2);
    let [bar_area] = bar_layout.areas(bottom_area);

    let rows = [
        Row::new(vec![
            "MAC", 
            "IP", 
            "Identity", 
            "Version", 
            "Interface", 
            "Uptime", 
            "ID", 
            "Board"
        ]),
        Row::new(vec![
            "B8:69:F4:BE:3C:29", 
            "196.223.31.42", 
            "GNS_SAIX_TERACO_CCR01", 
            "6.47.10(long-term)",
            "sfp5",
            "1295h10m5s", 
            "CCR1016-12S-1S+"
        ]),
        // Row::new(vec!["Row31", "Row32", "Row33"]),
    ];
    let widths = [
        Constraint::Length(18),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
    ];
    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title("List devices")
                .borders(Borders::ALL)
            )
        .row_highlight_style(Style::new().reversed())
        .column_spacing(2)
        .highlight_symbol(">>");

    let [table_area] = bar_layout.areas(top_area);

    frame.render_widget(table, table_area);
    frame.render_widget(bar, bar_area);
}

fn handle_events() -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => return Ok(true),
            // handle other key events
            _ => {}
        },
        // handle other events
        _ => {}
    }
    Ok(false)
}

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| {
        loop {
            terminal.draw(render)?;
            if handle_events()? {
                break Ok(());
            }
        }
    })
}

