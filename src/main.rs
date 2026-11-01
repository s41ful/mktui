use Constraint::{Fill, Length, Min, Percentage};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use mndp_rs::{Device, Listener, MndpError};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect, Position},
    text::{Line, Span},
    backend::CrosstermBackend,
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, Cell, Row, TableState, Table,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame, Terminal,
};

use std::io::{self};
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::ui::theme::{Theme, ThemePicker};
pub mod ui;

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
    theme_picker: Option<ThemePicker>,
    theme: Theme,
    status_bar: String,
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
            theme: Theme::winbox_dark(),
            theme_picker: None,
            status_bar: String::from("Listening from 0.0.0.0:5678"),
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

    fn handle_login_event_code(&mut self, code: KeyCode) -> std::io::Result<bool> {
        if self.input_mode {
            if event::poll(Duration::from_millis(50))? {
                self.handle_input_mode();
            }

            Ok(false)
        } else {
            match code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('i') => self.input_mode = true,
                    KeyCode::Down | KeyCode::Char('j') => self.next(),
                    KeyCode::Up | KeyCode::Char('k') => self.prev(),
                    KeyCode::Char('m') => self.state = AppState::Main,
                    // handle other key events
                    _ => {}
                }
            Ok(false)
        }
    }

    fn handle_main_event_code(&mut self, code: KeyCode) -> std::io::Result<bool> {
        match code {
            KeyCode::Char('g') => self.list_state.select_first(),
            KeyCode::Char('G') => self.list_state.select_last(),
            KeyCode::Down | KeyCode::Char('j') => self.list_state.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.select_previous(),
            KeyCode::Char('b') => self.state = AppState::Login,
            _ => {}
        }

        Ok(false)
    }

    pub fn handle_events(&mut self) -> std::io::Result<bool> {
        if self.input_mode {
            self.handle_input_mode();
            return Ok(false)
        }

        if let Event::Key(key) = event::read()? {
            // Windows reports both press and release for every keystroke; without
            // this each key would act twice.
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            } 

            match (key.code, key.modifiers) {
                (KeyCode::Char('t'), _) => { self.open_theme_picker() }
                (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    self.mndp_listener.stop_ctrl.store(true, std::sync::atomic::Ordering::SeqCst);
                    return Ok(true)
                }
                _ => {
                    if self.theme_picker.is_some() {
                        match (key.code, key.modifiers) {
                            (KeyCode::Esc, _) => self.cancel_theme(),
                            (KeyCode::Enter, _) => self.confirm_theme(),
                            (KeyCode::Char('p'), KeyModifiers::CONTROL) | 
                                (KeyCode::Up, _) => self.theme_picker_move(-1),
                            (KeyCode::Char('n'), KeyModifiers::CONTROL) | 
                                (KeyCode::Down, _) => self.theme_picker_move(1),
                            (KeyCode::PageUp, _) => self.theme_picker_move(-10),
                            (KeyCode::PageDown, _) => self.theme_picker_move(10),
                            (KeyCode::Home, _) => self.theme_picker_move(isize::MIN / 2),
                            (KeyCode::End, _) => self.theme_picker_move(isize::MAX / 2),
                            (KeyCode::Backspace, _) => {
                                self.theme_picker_edit_query(|q| {
                                    q.pop();
                                });
                            }
                            (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => {
                                self.theme_picker_edit_query(|q| q.push(c));
                            }
                            _ => {}
                        }

                        return Ok(false)
                    }

                    match self.state {
                        AppState::Main => return self.handle_main_event_code(key.code),
                        AppState::Login => return self.handle_login_event_code(key.code),
                        AppState::Detail => return Ok(false),
                    }
                }
            }

            Ok(false)

        } else {
            return Ok(false)
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
        f.render_widget(Paragraph::new(String::from(&self.status_bar)), status_area);
        f.render_widget(Block::bordered().style(self.theme.border), config_area);

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

                let uptime = device.uptime.as_secs();
                let uptime_readable: String;

                if uptime < 86400 {
                    if uptime % 86400 / 3600 == 0 {
                        uptime_readable = format!("{}m", uptime % 86400 % 3600)
                    } else {
                        uptime_readable = format!("{}h{}m", uptime % 86400 / 3600, uptime % 86400 % 3600 / 60)
                    }
                } else {
                    uptime_readable = format!("{}d{}h{}m", uptime / 86400, uptime % 86400 / 3600, uptime % 86400 % 3600 / 60)
                }

                cells.push(Cell::from(device.mac_address.to_string()));
                cells.push(Cell::from(device.ipv4_address.to_string()));
                cells.push(Cell::from(String::from(&device.identity)));
                cells.push(Cell::from(String::from(&device.version)));
                cells.push(Cell::from(String::from(&device.interface_name)));
                cells.push(Cell::from(uptime_readable));
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
        f.render_widget(Paragraph::new(String::from(&self.status_bar)).style(self.theme.muted_text), status_area);
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
        if self.theme_picker.is_some() {
            Self::render_theme_modal(f, self);
        }
    }

    /// Open the theme picker.
    ///
    /// This replaced cycling. With one built-in theme and 238 bundled base16 schemes,
    /// stepping through them one keypress at a time is not a way to find anything.
    pub fn open_theme_picker(&mut self) {
        let entries = crate::ui::theme::catalogue();
        let selected = entries
            .iter()
            .position(|e| e.id == self.theme.id)
            .unwrap_or(0);

        self.theme_picker = Some(ThemePicker {
            entries,
            query: String::new(),
            selected,
            offset: std::cell::Cell::new(0),
            original: self.theme.id.clone(),
        });
    }
        /// Apply whatever the picker is currently highlighting.
    ///
    /// Called on every movement so the choice is previewed against real data rather than
    /// a swatch — the point of a theme is how the tables look under it.
    pub fn preview_highlighted_theme(&mut self) {
        if let Some(id) = self.theme_picker.as_ref().and_then(|p| p.current()) {
            if id != self.theme.id {
                self.theme = crate::ui::theme::load(&id);
            }
        }
    }

    pub fn theme_picker_move(&mut self, delta: isize) {
        let Some(picker) = self.theme_picker.as_mut() else {
            return;
        };
        let len = picker.matches().len();
        if len == 0 {
            return;
        }
        let last = len - 1;
        picker.selected = match delta {
            d if d < 0 => picker.selected.saturating_sub(d.unsigned_abs()),
            d => (picker.selected + d as usize).min(last),
        };
        self.preview_highlighted_theme();
    }

    pub fn theme_picker_edit_query(&mut self, edit: impl FnOnce(&mut String)) {
        let Some(picker) = self.theme_picker.as_mut() else {
            return;
        };
        edit(&mut picker.query);
        // The list just changed under the cursor.
        picker.selected = 0;
        picker.offset.set(0);
        self.preview_highlighted_theme();
    }

    /// Keep the highlighted theme and close.
    pub fn confirm_theme(&mut self) {
        if let Some(picker) = self.theme_picker.take() {
            let chosen = self.theme.id.clone();
            self.status_bar = format!("Theme: {}", self.theme.name);
            let _ = picker;
            // // Remember it, so the choice survives a restart.
            // if let Err(e) = crate::config::save_theme(&chosen) {
            //     self.status_bar = format!("Theme: {} (not saved: {e})", self.theme.name);
            // }
        }
    }

    /// Close without keeping the preview.
    pub fn cancel_theme(&mut self) {
        if let Some(picker) = self.theme_picker.take() {
            self.theme = crate::ui::theme::load(&picker.original);
        }
    }

    /// The theme picker.
    ///
    /// A list rather than the cycle this replaced: 239 themes is not something to step
    /// through one keypress at a time. Typing filters, and every movement applies the
    /// highlighted theme to the screen behind the modal, so the choice is judged against real
    /// tables instead of a swatch.
    pub fn render_theme_modal(f: &mut Frame, app: &App) {
        let Some(picker) = app.theme_picker.as_ref() else {
            return;
        };
        let t = &app.theme;

        let area = Self::centered_rect(60, 70, f.area());
        f.render_widget(Clear, area);

        let matches = picker.matches();
        let selected = picker.selected.min(matches.len().saturating_sub(1));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(t.border_focus)
            .title(Span::styled(
                    format!(" 🎨 Theme ({} of {}) ", matches.len(), picker.entries.len()),
                    t.title,
            ));
        let inner = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // search
                Constraint::Min(3),    // list
                Constraint::Length(1), // footer
            ])
            .split(inner);

        let search = if picker.query.is_empty() {
            Span::styled(" Type to search…", t.muted_text)
        } else {
            Span::styled(format!(" {}█", picker.query), t.warning)
        };
        f.render_widget(Paragraph::new(Line::from(search)), chunks[0]);

        let rows = chunks[1].height as usize;
        let offset = Self::visible_offset(picker.offset.get(), selected, rows, matches.len());
        picker.offset.set(offset);

        let items: Vec<ListItem> = matches
            .iter()
            .map(|entry| {
                let current = entry.id == t.id;
                let marker = if current { "● " } else { "  " };
                ListItem::new(Line::from(vec![
                        Span::styled(marker, t.success),
                        Span::styled(format!("{:<28}", entry.name), t.normal_text),
                        Span::styled(format!("{:<10}", entry.source.label()), t.muted_text),
                        Span::styled(entry.author.clone(), t.muted_text),
                ]))
            })
        .collect();

        let mut state = ListState::default()
            .with_offset(offset)
            .with_selected(Some(selected));

        f.render_stateful_widget(
            List::new(items).highlight_style(t.selected_row),
            chunks[1],
            &mut state,
        );

        if matches.len() > rows {
            let mut scrollbar_state = ScrollbarState::new(matches.len())
                .position(offset)
                .viewport_content_length(rows);
            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .style(t.border),
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }

        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                        " ↑↓ move · Enter keep · Esc cancel · Backspace edit ",
                        t.muted_text,
            ))),
            chunks[2],
        );
    }

    /// Smallest scroll that brings `selected` into view. Mirrors the tables' rule.
    fn visible_offset(current: usize, selected: usize, viewport: usize, total: usize) -> usize {
        if viewport == 0 {
            return 0;
        }
        let max_offset = total.saturating_sub(viewport);
        let mut offset = current.min(max_offset);
        if selected < offset {
            offset = selected;
        } else if selected >= offset + viewport {
            offset = selected + 1 - viewport;
        }
        offset.min(max_offset)
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(vertical[1])[1]
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
                        Ok(device) => {
                            if !app.devices.contains(&device) {
                                app.devices.push(device);
                            }
                        },
                        Err(err) => {
                            match err {
                                MndpError::TimedOut => {
                                    app.status_bar = String::from("listener timed out press 'r' ro refresh");
                                    continue
                                }
                                _ => eprintln!("mndp listener error: {:?}", err),
                            }
                        }
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
