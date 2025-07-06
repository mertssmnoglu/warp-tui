use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

use warp_tui::{WarpClient, WarpInfo, WarpStatus};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    /// Warp client for executing commands
    warp_client: WarpClient,
    /// Current warp information
    warp_info: WarpInfo,
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
            self.handle_crossterm_events()?;
        }
        Ok(())
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
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let title = Line::from("Cloudflare WARP TUI").bold().blue().centered();

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
            "Cloudflare WARP Terminal UI\n\n\
            Status: {}\n\
            {}\n\
            Account Type: {}\n\
            WARP Enabled: {}\n\
            Gateway Enabled: {}\n\n\
            Controls:\n\
            - Press 'c' to connect\n\
            - Press 'd' to disconnect\n\
            - Press 'r' to refresh status\n\
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
            }
        );

        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .style(ratatui::style::Style::default().fg(status_color)),
            frame.area(),
        )
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('c') | KeyCode::Char('C')) => self.handle_connect(),
            (_, KeyCode::Char('d') | KeyCode::Char('D')) => self.handle_disconnect(),
            (_, KeyCode::Char('r') | KeyCode::Char('R')) => self.handle_refresh(),
            // Add other key handlers here.
            _ => {}
        }
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

    /// Handle refresh status command
    fn handle_refresh(&mut self) {
        self.update_warp_status();
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
