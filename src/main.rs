use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
use std::time::{Duration, Instant};

use warp_tui::{WarpClient, WarpInfo, WarpStatus};

const AVAILABLE_MODES: &[&str] = &["doh", "dot", "warp+doh", "warp+dot"];

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    /// Warp client for executing commands
    warp_client: WarpClient,
    /// Current warp information
    warp_info: WarpInfo,
    /// Current refresh interval in milliseconds
    refresh_interval_ms: u64,
    /// Last refresh time
    last_refresh: Instant,
    /// Mode selection state
    mode_selection: Option<ListState>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: false,
            warp_client: WarpClient::default(),
            warp_info: WarpInfo::default(),
            refresh_interval_ms: 1000,
            last_refresh: Instant::now(),
            mode_selection: None,
        }
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;

        // Initialize warp status
        self.update_warp_status();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            // Check if we need to auto-refresh
            if self.should_auto_refresh() {
                self.update_warp_status();
            }

            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Check if it's time to auto-refresh
    fn should_auto_refresh(&self) -> bool {
        let refresh_interval = Duration::from_millis(self.refresh_interval_ms);
        self.last_refresh.elapsed() >= refresh_interval
    }

    /// Get current refresh interval in milliseconds
    fn current_refresh_interval(&self) -> u64 {
        self.refresh_interval_ms
    }

    /// Update the warp status information
    fn update_warp_status(&mut self) {
        match self.warp_client.get_status_sync() {
            Ok(info) => self.warp_info = info,
            Err(_) => {
                // If we can't get status, reset to default
                self.warp_info = WarpInfo::default();
            }
        }
        // Reset the refresh timer whenever we update status
        self.last_refresh = Instant::now();
    }

    /// Handle connect command
    fn handle_connect(&mut self) {
        // Execute connect command synchronously
        match self.warp_client.connect_sync() {
            Ok(_) => {
                // Connection initiated, update status
                self.update_warp_status();
            }
            Err(_) => {
                // Connection failed, still update status to show current state
                self.update_warp_status();
            }
        }
    }

    /// Handle disconnect command  
    fn handle_disconnect(&mut self) {
        // Execute disconnect command synchronously
        match self.warp_client.disconnect_sync() {
            Ok(_) => {
                // Disconnection initiated, update status
                self.update_warp_status();
            }
            Err(_) => {
                // Disconnection failed, still update status to show current state
                self.update_warp_status();
            }
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    /// Handle mode selection
    fn handle_mode_selection(&mut self) {
        // Toggle mode selection UI
        if self.mode_selection.is_none() {
            let mut state = ListState::default();

            // Find the index of current mode
            let current_mode = self
                .warp_info
                .mode
                .as_ref()
                .map(|m| m.to_string().to_lowercase());
            let selected_idx = match current_mode {
                Some(mode) => AVAILABLE_MODES
                    .iter()
                    .position(|&m| m == mode.replace("+", "+")),
                None => None,
            }
            .unwrap_or(0);

            state.select(Some(selected_idx));
            self.mode_selection = Some(state);
        } else {
            self.mode_selection = None;
        }
    }

    /// Handle mode selection key
    fn handle_mode_select(&mut self) {
        if let Some(list_state) = &mut self.mode_selection {
            if let Some(selected) = list_state.selected() {
                let mode = AVAILABLE_MODES[selected];
                if let Ok(()) = self.warp_client.set_mode_sync(mode) {
                    self.update_warp_status();
                }
                self.mode_selection = None;
            }
        }
    }

    /// Handle selection movement up
    fn select_previous(&mut self) {
        if let Some(list_state) = &mut self.mode_selection {
            let current = list_state.selected().unwrap_or(0);
            let next = if current == 0 {
                AVAILABLE_MODES.len() - 1
            } else {
                current - 1
            };
            list_state.select(Some(next));
        }
    }

    /// Handle selection movement down
    fn select_next(&mut self) {
        if let Some(list_state) = &mut self.mode_selection {
            let current = list_state.selected().unwrap_or(0);
            let next = if current >= AVAILABLE_MODES.len() - 1 {
                0
            } else {
                current + 1
            };
            list_state.select(Some(next));
        }
    }

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Main content/Mode selection
            ])
            .split(area);

        // Render the title
        let title = Line::from("Cloudflare WARP TUI").bold().blue().centered();
        frame.render_widget(
            Paragraph::new(title)
                .block(Block::bordered())
                .style(Style::default()),
            chunks[0],
        );

        // Show mode selection if active
        if let Some(mode_selection) = &mut self.mode_selection {
            // Create mode list items
            let mode_items: Vec<ListItem> = AVAILABLE_MODES
                .iter()
                .map(|mode| ListItem::new(*mode))
                .collect();

            let mode_list = List::new(mode_items)
                .block(Block::bordered().title("Select Mode"))
                .style(Style::default())
                .highlight_style(Style::default().reversed());

            // Render the mode selection UI
            frame.render_stateful_widget(mode_list, chunks[1], mode_selection);
            return;
        }

        let status_color = match self.warp_info.status {
            WarpStatus::Connected => ratatui::style::Color::Green,
            WarpStatus::Disconnected => ratatui::style::Color::Red,
            WarpStatus::Connecting | WarpStatus::Disconnecting => ratatui::style::Color::Yellow,
            WarpStatus::Unknown => ratatui::style::Color::Gray,
        };

        let mode_text = match &self.warp_info.mode {
            Some(mode) => format!("Mode: {}", mode),
            None => "Mode: N/A".to_string(),
        };

        let text = format!(
            "Status: {}\n\
            {}\n\
            Account Type: {}\n\
            WARP Enabled: {}\n\
            Gateway Enabled: {}\n\
            Auto-refresh: {}ms\n\n\
            Controls:\n\
            - Press 'c' to connect\n\
            - Press 'd' to disconnect\n\
            - Press 'r' to refresh status\n\
            - Press 'm' to change mode\n\
            - Use Up/Down arrows to navigate mode selection\n\
            - Press 'Enter' to select mode\n\
            - Press 'Esc' to cancel mode selection\n\
            - Press 'Esc', 'Ctrl-C' or 'q' to quit",
            self.warp_info.status,
            mode_text,
            self.warp_info.account_type.as_deref().unwrap_or("N/A"),
            if self.warp_info.warp_enabled {
                "Yes"
            } else {
                "No"
            },
            if self.warp_info.gateway_enabled {
                "Yes"
            } else {
                "No"
            },
            self.current_refresh_interval()
        );

        // Render main content
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered())
                .style(Style::default().fg(status_color)),
            chunks[1],
        );
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        // Poll for events with a small timeout to avoid blocking
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            // Global control keys
            (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),

            // Mode selection specific keys
            _ if self.mode_selection.is_some() => match key.code {
                KeyCode::Esc => self.mode_selection = None,
                KeyCode::Up => self.select_previous(),
                KeyCode::Down => self.select_next(),
                KeyCode::Enter => self.handle_mode_select(),
                _ => {}
            },

            // Normal mode keys
            (_, KeyCode::Esc | KeyCode::Char('q')) => self.quit(),
            (_, KeyCode::Char('c') | KeyCode::Char('C')) => self.handle_connect(),
            (_, KeyCode::Char('d') | KeyCode::Char('D')) => self.handle_disconnect(),
            (_, KeyCode::Char('m') | KeyCode::Char('M')) => self.handle_mode_selection(),
            _ => {}
        }
    }
}
