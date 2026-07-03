use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::sync::{Arc, Mutex};
use tui_input::backend::crossterm::EventHandler;
use std::time::Duration;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::{self, Frame, Terminal};
use std::io;
use tui_input::{Input};

struct App {
    list_input: [Arc<Mutex<Input>>; 3],
    index_input: usize,
    current_input: Arc<Mutex<Input>>,
    input_mode: bool,
    table_state: TableState,
    table_items: Vec<Vec<&'static str>>,
}

const FOCUSED_BORDER_STYLE: Style = Style::new()
    .fg(Color::Blue);
const INPUT_WIDGET_STYLE: Style = Style::new().fg(Color::LightCyan);
const SELECTED_TABLE_STYLE: Style = Style::new()
            .bg(Color::Blue) // background
            .fg(Color::White) // foreground
            .add_modifier(Modifier::BOLD);
const HEADER_STYLE: Style = Style::new()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

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

        let address_input = Arc::new(Mutex::new(Input::default()));
        let username_input = Arc::new(Mutex::new(Input::default()));
        let password_input = Arc::new(Mutex::new(Input::default()));

        App {
            table_state: state,
            current_input: Arc::clone(&address_input),
            list_input: [Arc::clone(&address_input), Arc::clone(&username_input), Arc::clone(&password_input)],
            index_input: 0,
            input_mode: false,
            table_items: items,
        }
    }

    pub fn next(&mut self) {
        self.table_state.select_next();
    }

    pub fn prev(&mut self) {
        self.table_state.select_previous();
    }

    fn next_input(&mut self) {
        if self.index_input < 2 {
            self.index_input+= 1;
            self.current_input = Arc::clone(&self.list_input[self.index_input])
        }
    }

    fn prev_input(&mut self) {
        if self.index_input > 0 {
            self.index_input-= 1;
            self.current_input = Arc::clone(&self.list_input[self.index_input])
        }
    }

    fn new_input_widget(input_value: &str, visual_scroll: usize) -> Paragraph<'_> {
        Paragraph::new(input_value)
            .style(INPUT_WIDGET_STYLE)
            .scroll((0, visual_scroll as u16))
            .block(Block::default().borders(Borders::ALL))

    }

    fn handle_input_mode(&mut self) {
        if event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Enter => {
                    }
                    KeyCode::Esc => {
                        self.input_mode = false; 
                    }
                    KeyCode::Tab | KeyCode::Down => {
                        self.next_input();
                    }
                    KeyCode::Up => {
                        self.prev_input();
                    }
                    _ => {
                        let mut current_input = self.current_input.lock().unwrap();
                        current_input.handle_event(&Event::Key(key));
                    }
                }
            }
        }
    }

    pub fn handle_events(&mut self) -> std::io::Result<bool> {
        if self.input_mode {
            if event::poll(Duration::from_millis(50))? {
                self.handle_input_mode();
            }

            Ok(false)
        } else {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('i') => self.input_mode = true,
                    KeyCode::Down | KeyCode::Char('j') => self.next(),
                    KeyCode::Up | KeyCode::Char('k') => self.prev(),
                    KeyCode::Char('a') => {
                        self.table_items.push(
                            vec![
                            "B8:69:F4:BE:3C:29",
                            "196.223.31.42",
                            "GNS_SAIX_TERACO_CCR01",
                            "6.47.10(long-term)",
                            "sfp5",
                            "1295h10m5s",
                            "CCR1016-12S-1S+",
                            ],
                        );
                    }
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
        let vertical2 = Layout::vertical([Fill(1), Fill(2)]);
        let [top_area, bottom_area] = vertical2.areas(main_area);
        let input_vertical = Layout::vertical([Length(3); 3])
            .margin(2);

        let top_area_style = if self.input_mode {
            FOCUSED_BORDER_STYLE
        } else { Style::default() };

        let bottom_area_style = if !self.input_mode {
            FOCUSED_BORDER_STYLE
        } else { Style::default() };

        let [message_input_area, input_area] = Layout::horizontal([Percentage(20), Fill(1)])
            .areas(top_area);
        let [message_input_area] = Layout::vertical([Fill(1)]).margin(1).areas(message_input_area);

        let [message_address_area, message_username_area, message_password_area] = input_vertical.areas(message_input_area);

        let [input_address_area, input_username_area, input_password_area] = input_vertical.areas(input_area);

        let width = input_address_area.width; 

        let [ref_address_input, ref_username_input, ref_password_input] = 
            self.list_input
            .each_ref()
            .map(|arc| arc.lock().unwrap());

        let address_scroll = ref_address_input.visual_scroll(width as usize);
        let username_scroll = ref_username_input.visual_scroll(width as usize);
        let password_scroll = ref_password_input.visual_scroll(width as usize);

        let message_address_input = Paragraph::new("MAC/IP Address\t:");
        let message_username_input = Paragraph::new("Username\t:");
        let message_password_input = Paragraph::new("Password\t:");

        let address_input = App::new_input_widget(ref_address_input.value(), address_scroll);
        let username_input = App::new_input_widget(ref_username_input.value(), username_scroll);
        let password_input = App::new_input_widget(ref_password_input.value(), password_scroll);

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
            .style(HEADER_STYLE)
            .height(1);

        let rows: Vec<Row> = self
            .table_items
            .iter()
            .map(|item| {
                let cells: Vec<Cell> = item.iter().map(|c| Cell::from(*c)).collect();
                Row::new(cells).height(1)
            })
        .collect();

        let widths = [
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(25),
            Constraint::Length(20),
            Constraint::Length(9),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                .borders(Borders::ALL)
                .border_style(bottom_area_style)
                .title("List Devices"))
            .row_highlight_style(SELECTED_TABLE_STYLE)
            .highlight_symbol(">> ");

        f.render_widget(Block::bordered().title("Title Bar"), title_area);
        f.render_widget(Block::bordered().title("Status Bar"), status_area);
        f.render_widget(
            Block::bordered()
            .title("Connect to")
            .border_style(top_area_style),
            top_area);

        // Render Inputs
        f.render_widget(address_input, input_address_area);
        f.render_widget(username_input, input_username_area);
        f.render_widget(password_input, input_password_area);

        // Set the cursor position
        if self.input_mode {
            let input_position = match self.index_input {
                0 => input_address_area,
                1 => input_username_area,
                2 => input_password_area,
                _ => todo!()
            };

            let input_scroll = match self.index_input {
                0 => address_scroll,
                1 => username_scroll,
                2 => password_scroll,
                _ => todo!(),
            };

            drop(ref_address_input);
            drop(ref_username_input);
            drop(ref_password_input);

            let ref_current_input = self.current_input.lock().unwrap();

            f.set_cursor_position(Position {
                x: (input_position.x + 1 + (ref_current_input.cursor() - input_scroll) as u16)
                    .min(input_position.x + input_position.width - 2),
                    y: input_position.y + 1,
            });

        }

        // Render message inputs
        f.render_widget(message_address_input, message_address_area);
        f.render_widget(message_username_input, message_username_area);
        f.render_widget(message_password_input, message_password_area);

        // Render the bottom table
        f.render_stateful_widget(table, bottom_area, &mut self.table_state);
    }
}

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    let mut app = App::new();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| app.render(f))?;

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
