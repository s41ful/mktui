use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use tui_input::backend::crossterm::EventHandler;
use std::time::Duration;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Position};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::{self, Frame, Terminal};
use std::io;
use tui_input::{Input};

struct App {
    address_input: Input,
    username_input: Input,
    password_input: Input,
    input_mode: bool,
    state: TableState,
    items: Vec<Vec<&'static str>>,
}

impl App {
    fn new() -> App {
        let mut state = TableState::default();
        state.select(Some(0));

        let items = vec![
            vec![
                "B8:69:F4:BE:3C:29",
                "196.223.31.42",
                "GNS_SAIX_TERACO_CCR01",
                "6.47.10(long-term)",
                "sfp5",
                "1295h10m5s",
                "CCR1016-12S-1S+",
            ],
            vec![
                "B8:69:F4:BE:3C:29",
                "196.223.31.42",
                "GNS_SAIX_TERACO_CCR01",
                "6.47.10(long-term)",
                "sfp5",
                "1295h10m5s",
                "CCR1016-12S-1S+",
            ],
            vec![
                "B8:69:F4:BE:3C:29",
                "196.223.31.42",
                "GNS_SAIX_TERACO_CCR01",
                "6.47.10(long-term)",
                "sfp5",
                "1295h10m5s",
                "CCR1016-12S-1S+",
            ],
        ];

        App {
            state: state,
            address_input: Input::default(),
            username_input: Input::default(),
            password_input: Input::default(),
            input_mode: false,
            items: items,
        }
    }

    pub fn next(&mut self) {
        self.state.select_next();
    }

    pub fn prev(&mut self) {
        self.state.select_previous();
    }

    pub fn handle_events(&mut self) -> std::io::Result<bool> {
        if self.input_mode {
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Enter => {
                            // Aksi saat user menekan Enter (misal: simpan/kirim pesan)
                            // break; untuk keluar aplikasi
                        }
                        KeyCode::Esc => {
                            self.input_mode = false; // Keluar dari loop
                        }
                        _ => {
                            // Berikan event key ke state tui-input agar diproses
                            // (menyisipkan karakter, menghapus spasi/backspace, dll)
                            self.address_input.handle_event(&Event::Key(key));
                        }
                    }
                }
            }

            Ok(false)
        } else {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('i') => {
                        self.input_mode = true;
                    }
                    KeyCode::Down | KeyCode::Char('j') => self.next(),
                    KeyCode::Up | KeyCode::Char('k') => self.prev(),
                    // handle other key events
                    _ => {}
                },
                // handle other events
                _ => {}
            }
            Ok(false)
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();
        use Constraint::{Percentage, Fill, Length, Min};

        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [title_area, main_area, status_area] = vertical.areas(area);
        let horizontal = Layout::vertical([Fill(1), Fill(2)]);
        let [top_area, bottom_area] = horizontal.areas(main_area);

        let [message_input_area, input_area] = Layout::horizontal([Percentage(20), Fill(1)])
            .areas(top_area);

        let [message_address_area, message_username_area, message_password_area] = Layout::vertical([Length(3); 3])
            .margin(2)
            .areas(message_input_area);

        let [input_address_area, input_username_area, input_password_area] = Layout::vertical([Length(3); 3])
            .margin(1)
            .areas(input_area);

        // let chunks = Layout::default()
        //     .direction(Direction::Vertical)
        //     .margin(1)
        //     .constraints([Constraint::Length(3)].as_ref())
        //     .split(input_area);

        // Tampilkan nilai input ke dalam Widget Paragraph
        let width = message_address_area.width; // Kurangi border
        let address_scroll = self.address_input.visual_scroll(width as usize);
        let username_scroll = self.username_input.visual_scroll(width as usize);
        let password_scroll = self.username_input.visual_scroll(width as usize);

        let message_address_input = Paragraph::new("MAC/IP Address\t:")
            .style(Style::default()
            .fg(Color::Blue));

        let message_username_input = Paragraph::new("Username\t:")
            .style(Style::default()
            .fg(Color::Blue));

        let message_password_input = Paragraph::new("Password\t:")
            .style(Style::default()
            .fg(Color::Blue));

        let address_input = Paragraph::new(self.address_input.value())
            .style(Style::default().fg(Color::LightCyan))
            .scroll((0, address_scroll as u16))
            .block(Block::default().borders(Borders::ALL));

        let username_input = Paragraph::new(self.username_input.value())
            .style(Style::default().fg(Color::LightCyan))
            .scroll((0, username_scroll as u16))
            .block(Block::default().borders(Borders::ALL));

        let password_input = Paragraph::new(self.username_input.value())
            .style(Style::default().fg(Color::LightCyan))
            .scroll((0, password_scroll as u16))
            .block(Block::default().borders(Borders::ALL));
        
        // Highlighted style
        let selected_style = Style::default()
            .bg(Color::Blue) // background
            .fg(Color::White) // foreground
            .add_modifier(Modifier::BOLD);

        let header_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        // Header component for mikrotik discovery
        let header = Row::new(vec![
            "MAC",
            "IP",
            "Identity",
            "Version",
            "Interface",
            "Uptime",
            "ID",
            "Board",
        ])
        .style(header_style)
        .height(1);

        let rows: Vec<Row> = self
            .items
            .iter()
            .map(|item| {
                let cells: Vec<Cell> = item.iter().map(|c| Cell::from(*c)).collect();
                Row::new(cells).height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("List Devices"))
            .row_highlight_style(selected_style)
            .highlight_symbol(">> ");

        f.render_widget(Block::bordered().title("Title Bar"), title_area);
        f.render_widget(Block::bordered().title("Status Bar"), status_area);
        f.render_widget(Block::bordered().title("Connect to"), top_area);
        
        // Render Inputs
        f.render_widget(address_input, input_address_area);
        f.render_widget(username_input, input_username_area);
        f.render_widget(password_input, input_password_area);

        // Set the cursor position
        f.set_cursor_position(Position {
            x: (input_address_area.x + 1 + (self.address_input.cursor() - address_scroll) as u16)
                .min(input_address_area.x + input_address_area.width - 2),
            y: input_address_area.y + 1,
        });

        // Render message inputs
        f.render_widget(message_address_input, message_address_area);
        f.render_widget(message_username_input, message_username_area);
        f.render_widget(message_password_input, message_password_area);

        // Render the bottom table
        f.render_stateful_widget(table, bottom_area, &mut self.state);
    }
}

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    let mut app = App::new();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main Loop Aplikasi
    loop {
        // Render UI setiap frame
        terminal.draw(|f| app.render(f))?;
        // 2. Tangkap Event Keyboard
        if app.handle_events()? {
            break;
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
