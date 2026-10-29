use Constraint::{Fill, Length, Min, Percentage};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use mndp_rs::{Device, Listener};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, List, ListState, Paragraph, Row, Table, TableState};
use ratatui::{self, Frame, Terminal};
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub border: Style,
    pub border_focus: Style,
    pub title: Style,
    pub header_cell: Style,
    pub selected_row: Style,
    pub normal_text: Style,
    pub muted_text: Style,
    pub accent: Style,
    pub success: Style,
    pub warning: Style,
    pub danger: Style,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            name: "WinBox Dark".to_string(),
            border: Style::default().fg(Color::Rgb(70, 82, 100)),
            border_focus: Style::default().fg(Color::Rgb(90, 165, 230)),
            title: Style::default()
                .fg(Color::Rgb(105, 190, 245))
                .add_modifier(Modifier::BOLD),
                header_cell: Style::default()
                    .fg(Color::Rgb(220, 228, 235))
                    .bg(Color::Rgb(32, 42, 58))
                    .add_modifier(Modifier::BOLD),
                    selected_row: Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(45, 90, 150))
                        .add_modifier(Modifier::BOLD),
                        normal_text: Style::default().fg(Color::Rgb(210, 218, 225)),
                        muted_text: Style::default().fg(Color::Rgb(125, 138, 155)),
                        accent: Style::default().fg(Color::Rgb(90, 180, 235)),
                        success: Style::default().fg(Color::Rgb(70, 190, 120)),
                        warning: Style::default().fg(Color::Rgb(235, 180, 70)),
                        danger: Style::default().fg(Color::Rgb(230, 80, 80)),
        }
    }
}

struct App {
    state: AppState,
    list_input: [Arc<Mutex<Input>>; 3],
    index_input: usize,
    current_input: Arc<Mutex<Input>>,
    input_mode: bool,
    table_state: TableState,
    list_state: ListState,
    list_items: Vec<&'static str>,
    mndp_listener: Listener,
    devices: Vec<Device>,
    theme: Theme,
}

enum AppState {
    Login,
    Main,
    Detail,
}


impl App {
    fn new() -> App {
        let mut state = TableState::default();
        let mut list_state = ListState::default();
        state.select(Some(0));
        list_state.select(Some(0));

        let address_input = Arc::new(Mutex::new(Input::default()));
        let username_input = Arc::new(Mutex::new(Input::default()));
        let password_input = Arc::new(Mutex::new(Input::default()));

        let list_items = vec![
            "/caps-man",
            "/console",
            "/ip",
            "/ipv6",
            "/interface",
            "/user",
            "/system",
            "/tool",
        ];

        App {
            state: AppState::Login,
            table_state: state,
            list_state: list_state,
            list_items: list_items,
            current_input: Arc::clone(&address_input),
            list_input: [
                Arc::clone(&address_input),
                Arc::clone(&username_input),
                Arc::clone(&password_input),
            ],
            index_input: 0,
            input_mode: false,
            mndp_listener: Listener::new(mndp_rs::MndpConfig {
                interface: None,
                timeout: Duration::from_secs(100),
            })
            .expect("creating mndp Listener error "),
            devices: Vec::new(),
            theme: Theme::new(),
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
            self.index_input += 1;
            self.current_input = Arc::clone(&self.list_input[self.index_input])
        }
    }

    fn prev_input(&mut self) {
        if self.index_input > 0 {
            self.index_input -= 1;
            self.current_input = Arc::clone(&self.list_input[self.index_input])
        }
    }

    fn new_input_widget(input_value: &str, visual_scroll: usize) -> Paragraph<'_> {
        Paragraph::new(input_value)
            .scroll((0, visual_scroll as u16))
            .block(Block::default().borders(Borders::ALL))
    }

    fn handle_input_mode(&mut self) {
        if event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Enter => {}
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

    fn handle_login_event(&mut self) -> std::io::Result<bool> {
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
                    KeyCode::Char('t') => self.state = AppState::Main,
                    // handle other key events
                    _ => {}
                },
                // handle other events
                _ => {}
            }
            Ok(false)
        }
    }

    fn handle_main_event(&mut self) -> std::io::Result<bool> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('g') => self.list_state.select_first(),
                KeyCode::Char('G') => self.list_state.select_last(),
                KeyCode::Down | KeyCode::Char('j') => self.list_state.select_next(),
                KeyCode::Up | KeyCode::Char('k') => self.list_state.select_previous(),
                KeyCode::Char('b') => self.state = AppState::Login,
                _ => {}
            },
            // handle other events
            _ => {}
        }
        Ok(false)
    }

    pub fn handle_events(&mut self) -> std::io::Result<bool> {
        match self.state {
            AppState::Login => self.handle_login_event(),
            AppState::Main => self.handle_main_event(),
            AppState::Detail => {
                todo!()
            }
        }
    }

    fn render_main(&mut self, f: &mut Frame) {
        let area = f.area();
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [title_area, main_area, status_area] = vertical.areas(area);
        let horizontal = Layout::horizontal([Fill(1), Fill(2)]);
        let [list_area, config_area] = horizontal.areas(main_area);

        let list_widget = List::new(self.list_items.clone())
            .highlight_style(self.theme.selected_row)
            .style(self.theme.border)
            .block(
                Block::default()
                .borders(Borders::ALL)
                .title("List configuration"),
            )
            .highlight_symbol(" > ");

        f.render_widget(
            Paragraph::new("mktui v0.1.0 - 192.168.1.2 (Mikrotik)").style(self.theme.title),
            title_area,
        );
        f.render_widget(Block::bordered().title("Status Bar"), status_area);
        f.render_widget(Block::bordered(), config_area);

        // Render list_state
        f.render_stateful_widget(list_widget, list_area, &mut self.list_state);
    }

    fn render_login(&mut self, f: &mut Frame) {
        let area = f.area();
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [title_area, main_area, status_area] = vertical.areas(area);
        let vertical2 = Layout::vertical([Fill(1), Fill(2)]);
        let [top_area, bottom_area] = vertical2.areas(main_area);
        let input_vertical = Layout::vertical([Length(3); 3]).margin(2);

        let top_area_style = if self.input_mode {
            self.theme.border_focus
        } else {
            self.theme.border
        };

        let bottom_area_style = if !self.input_mode {
            self.theme.border_focus
        } else {
            self.theme.border
        };

        let [message_input_area, input_area] =
            Layout::horizontal([Percentage(20), Fill(1)]).areas(top_area);
        let [message_input_area] = Layout::vertical([Fill(1)])
            .margin(1)
            .areas(message_input_area);

        let [
            message_address_area,
            message_username_area,
            message_password_area,
        ] = input_vertical.areas(message_input_area);

        let [input_address_area, input_username_area, input_password_area] =
            input_vertical.areas(input_area);

        let width = input_address_area.width;

        let [ref_address_input, ref_username_input, ref_password_input] =
            self.list_input.each_ref().map(|arc| arc.lock().unwrap());

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
            .style(self.theme.header_cell)
            .height(1);

        let rows: Vec<Row> = self
            .devices
            .iter()
            .map(|device| {
                let mut cells: Vec<Cell> = vec![];
                cells.push(Cell::from(device.mac_address.to_string()));
                cells.push(Cell::from(device.ipv4_address.to_string()));
                cells.push(Cell::from(String::from(&device.identity)));
                cells.push(Cell::from(String::from(&device.version)));
                cells.push(Cell::from(String::from(&device.interface_name)));
                cells.push(Cell::from("Skip duration"));
                cells.push(Cell::from(String::from(&device.software_id)));
                cells.push(Cell::from(String::from(&device.board)));

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
                .title("List Devices"),
            )
            .row_highlight_style(self.theme.selected_row)
            .highlight_symbol(">> ");

        f.render_widget(Paragraph::new("mktui v0.1.0").style(self.theme.title), title_area);
        f.render_widget(Paragraph::new("Status Bar").style(self.theme.muted_text), status_area);
        f.render_widget(
            Block::bordered()
            .title("Connect to")
            .border_style(top_area_style),
            top_area,
        );

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
                _ => todo!(),
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

    pub fn render(&mut self, f: &mut Frame) {
        match self.state {
            AppState::Login => self.render_login(f),
            AppState::Main => self.render_main(f),
            AppState::Detail => todo!(),
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    let mut app = App::new();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut device_rx = app.mndp_listener.start_discovery_stream();
    let sleep = time::sleep(Duration::from_millis(100));
    tokio::pin!(sleep);

    loop {
        terminal.draw(|f| app.render(f))?;

        tokio::select! {
            maybe_device = device_rx.recv() => {
                if let Some(result) = maybe_device {
                    match result {
                        Ok(device) => app.devices.push(device),
                        Err(_) => todo!(),
                    }
                }
            }
            _ = &mut sleep => {
                if app.handle_events()? {
                    drop(device_rx);
                    break;
                }
            }
        }
    }

    // restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
