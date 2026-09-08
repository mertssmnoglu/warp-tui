use color_eyre::Result;
use color_eyre::eyre::eyre;
use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io::{Write, stdout};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use warp_tui::{WarpClient, WarpInfo, WarpStatus, ensure_warp_cli};

const REPO_URL: &str = "github.com/mertssmnoglu/warp-tui";

const AVAILABLE_MODES: &[&str] = &[
    "warp",
    "doh",
    "warp+doh",
    "dot",
    "warp+dot",
    "proxy",
    "tunnel_only",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKind {
    AuthOrg,
    AuthToken,
    ConfirmUnenroll,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let binary = ensure_warp_cli().map_err(|e| eyre!(e))?;
    let mut terminal = ratatui::init();
    let _ = execute!(stdout(), EnableBracketedPaste);
    let result = App::with_client(WarpClient::with_binary(binary)).run(&mut terminal);
    let _ = execute!(stdout(), DisableBracketedPaste);
    ratatui::restore();
    result
}

#[derive(Debug)]
pub struct App {
    running: bool,
    warp_client: WarpClient,
    warp_info: WarpInfo,
    refresh_interval_ms: u64,
    last_refresh: Instant,
    mode_selection: Option<ListState>,
    force_disconnect: bool,
    force_disconnect_kills: u32,
    input: Option<InputKind>,
    input_buf: String,
    message: Option<String>,
    show_details: bool,
    paused_at: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        Self::with_client(WarpClient::default())
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    fn with_client(warp_client: WarpClient) -> Self {
        Self {
            running: false,
            warp_client,
            warp_info: WarpInfo::default(),
            refresh_interval_ms: 1000,
            last_refresh: Instant::now(),
            mode_selection: None,
            force_disconnect: false,
            force_disconnect_kills: 0,
            input: None,
            input_buf: String::new(),
            message: None,
            show_details: false,
            paused_at: None,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.running = true;
        self.update_warp_status();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            if self.should_auto_refresh() {
                self.update_warp_status();
                self.tick_force_disconnect();
            }

            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn should_auto_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= Duration::from_millis(self.refresh_interval_ms)
    }

    fn update_warp_status(&mut self) {
        match self.warp_client.get_status_sync() {
            Ok(info) => {
                let paused = info.status == WarpStatus::Disconnected
                    && info
                        .reason
                        .as_deref()
                        .is_some_and(|r| r.eq_ignore_ascii_case("paused"));
                if paused {
                    if self.paused_at.is_none() {
                        self.paused_at = Some(Instant::now());
                    }
                } else {
                    self.paused_at = None;
                }
                self.warp_info = info;
            }
            Err(e) => {
                self.warp_info = WarpInfo::default();
                self.paused_at = None;
                self.message = Some(e.to_string());
            }
        }
        self.last_refresh = Instant::now();
    }

    fn tick_force_disconnect(&mut self) {
        if !self.force_disconnect || !self.warp_info.status.is_active() {
            return;
        }
        match self.warp_client.disconnect_sync() {
            Ok(()) => self.force_disconnect_kills += 1,
            Err(e) => {
                self.message = Some(e.to_string());
                self.force_disconnect = false;
            }
        }
        self.update_warp_status();
    }

    fn handle_connect(&mut self) {
        self.force_disconnect = false;
        if let Err(e) = self.warp_client.connect_sync() {
            self.message = Some(e.to_string());
        } else {
            self.message = None;
        }
        self.update_warp_status();
    }

    fn handle_pause(&mut self) {
        self.force_disconnect = false;
        if let Err(e) = self.warp_client.disconnect_sync() {
            self.message = Some(e.to_string());
        } else {
            self.message = Some("Paused".into());
        }
        self.update_warp_status();
    }

    fn handle_disconnect(&mut self) {
        self.force_disconnect = true;
        self.force_disconnect_kills = 0;
        if let Err(e) = self.warp_client.disconnect_sync() {
            self.message = Some(e.to_string());
            self.force_disconnect = false;
        } else {
            self.force_disconnect_kills = 1;
            self.message = Some("Force disconnect on".into());
        }
        self.update_warp_status();
    }

    fn start_auth_input(&mut self) {
        self.input = Some(InputKind::AuthOrg);
        self.input_buf = std::env::var("WARP_TUI_ORG")
            .ok()
            .or_else(|| self.warp_info.organization.clone())
            .unwrap_or_default();
        self.message = Some("Enter Zero Trust team name".into());
    }

    fn start_token_input(&mut self) {
        self.input = Some(InputKind::AuthToken);
        self.input_buf.clear();
        self.message = Some("Paste the com.cloudflare.warp:// token".into());
    }

    fn start_unenroll_confirm(&mut self) {
        self.input = Some(InputKind::ConfirmUnenroll);
        self.input_buf.clear();
        self.message = Some("Delete registration? Enter confirm, Esc cancel".into());
    }

    fn copy_details(&mut self) {
        let mut parts = Vec::new();
        if let Some(org) = &self.warp_info.organization {
            parts.push(format!("Organization: {org}"));
        }
        if let Some(id) = &self.warp_info.account_id {
            parts.push(format!("Account ID: {id}"));
        }
        if let Some(id) = &self.warp_info.device_id {
            parts.push(format!("Device ID: {id}"));
        }
        if parts.is_empty() {
            self.message = Some("Nothing to copy".into());
            return;
        }
        match copy_to_clipboard(&parts.join("\n")) {
            Ok(()) => self.message = Some("Copied account/device ids".into()),
            Err(e) => self.message = Some(format!("Copy failed: {e}")),
        }
    }

    fn submit_input(&mut self) {
        let Some(kind) = self.input.take() else {
            return;
        };
        let value = std::mem::take(&mut self.input_buf);
        match kind {
            InputKind::AuthOrg => {
                let team = value.trim();
                if team.is_empty() {
                    self.message = Some("Team name was empty".into());
                    return;
                }
                match self.warp_client.enroll_sync(team) {
                    Ok(_) => {
                        self.message = Some(format!(
                            "Enrollment started for '{team}'. Finish login in the browser."
                        ));
                    }
                    Err(e) => {
                        self.message = Some(format!(
                            "{e} — press U to unenroll first if a registration already exists"
                        ));
                    }
                }
                self.update_warp_status();
            }
            InputKind::AuthToken => {
                let token = value.trim();
                if token.is_empty() {
                    self.message = Some("Token was empty".into());
                    return;
                }
                match self.warp_client.registration_token_sync(token) {
                    Ok(_) => self.message = Some("Registration token accepted".into()),
                    Err(e) => self.message = Some(e.to_string()),
                }
                self.update_warp_status();
            }
            InputKind::ConfirmUnenroll => {
                match self.warp_client.delete_registration_sync() {
                    Ok(()) => self.message = Some("Registration deleted".into()),
                    Err(e) => self.message = Some(e.to_string()),
                }
                self.update_warp_status();
            }
        }
    }

    fn cancel_input(&mut self) {
        self.input = None;
        self.input_buf.clear();
        self.message = None;
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn handle_mode_selection(&mut self) {
        if self.mode_selection.is_none() {
            let mut state = ListState::default();
            let current = self.warp_info.mode.as_ref().map(|m| m.cli_name());
            let selected_idx = current
                .and_then(|mode| AVAILABLE_MODES.iter().position(|&m| m == mode))
                .unwrap_or(0);
            state.select(Some(selected_idx));
            self.mode_selection = Some(state);
        } else {
            self.mode_selection = None;
        }
    }

    fn handle_mode_select(&mut self) {
        let Some(list_state) = &mut self.mode_selection else {
            return;
        };
        let Some(selected) = list_state.selected() else {
            return;
        };
        let mode = AVAILABLE_MODES[selected];
        match self.warp_client.set_mode_sync(mode) {
            Ok(()) => self.message = None,
            Err(e) => self.message = Some(e.to_string()),
        }
        self.update_warp_status();
        self.mode_selection = None;
    }

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

    fn status_color(&self) -> Color {
        match self.warp_info.status {
            WarpStatus::Connected => Color::Green,
            WarpStatus::Disconnected => Color::Red,
            WarpStatus::Connecting | WarpStatus::Disconnecting => Color::Yellow,
            WarpStatus::Unknown => Color::Gray,
        }
    }

    fn auto_connect_line(&self) -> String {
        let Some(mins) = self.warp_info.auto_connect_after_min else {
            return "Auto-connect: n/a".into();
        };
        if mins == 0 {
            return "Auto-connect: off".into();
        }
        let paused = self.warp_info.status == WarpStatus::Disconnected
            && self
                .warp_info
                .reason
                .as_deref()
                .is_some_and(|r| r.eq_ignore_ascii_case("paused"));
        if paused && let Some(started) = self.paused_at {
            let total = Duration::from_secs(u64::from(mins) * 60);
            if let Some(left) = total.checked_sub(started.elapsed()) {
                let secs = left.as_secs();
                return format!("Auto-connect in {}m {:02}s", secs / 60, secs % 60);
            }
            return "Auto-connect due".into();
        }
        format!("Auto-connect: {mins} min")
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new(Line::from("Cloudflare WARP").bold().blue().centered())
                .block(Block::bordered()),
            chunks[0],
        );

        if let Some(kind) = self.input {
            self.render_input(frame, chunks[1], kind);
        } else if self.mode_selection.is_some() {
            self.render_mode_select(frame, chunks[1]);
        } else if self.show_details {
            self.render_details(frame, chunks[1]);
        } else {
            self.render_dashboard(frame, chunks[1]);
        }

        let message = self.message.as_deref().unwrap_or("");
        let status_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(REPO_URL.len() as u16 + 1),
            ])
            .split(chunks[2]);
        frame.render_widget(
            Paragraph::new(message).style(Style::default().fg(Color::Yellow)),
            status_row[0],
        );
        frame.render_widget(
            Paragraph::new(REPO_URL)
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::DarkGray)),
            status_row[1],
        );

        let keys = if self.show_details {
            " y copy ids   i/esc close   q quit"
        } else {
            " c connect   p pause   d disconnect   a auth   i details   q quit"
        };
        frame.render_widget(
            Paragraph::new(keys).block(Block::bordered().title("Keys")),
            chunks[3],
        );
    }

    fn render_input(&self, frame: &mut Frame, area: Rect, kind: InputKind) {
        let title = match kind {
            InputKind::AuthOrg => "Zero Trust team name",
            InputKind::AuthToken => "Registration token",
            InputKind::ConfirmUnenroll => "Confirm unenroll",
        };
        let body = match kind {
            InputKind::ConfirmUnenroll => {
                "Press Enter to delete the current registration.\nPress Esc to cancel.".into()
            }
            _ => format!("{}\u{2588}", self.input_buf),
        };
        frame.render_widget(
            Paragraph::new(body)
                .block(Block::bordered().title(title))
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn render_mode_select(&mut self, frame: &mut Frame, area: Rect) {
        let Some(mode_selection) = &mut self.mode_selection else {
            return;
        };
        let mode_items: Vec<ListItem> = AVAILABLE_MODES
            .iter()
            .map(|mode| ListItem::new(*mode))
            .collect();
        let mode_list = List::new(mode_items)
            .block(Block::bordered().title("Select Mode"))
            .highlight_style(Style::default().reversed());
        frame.render_stateful_widget(mode_list, area, mode_selection);
    }

    fn render_dashboard(&self, frame: &mut Frame, area: Rect) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(9), Constraint::Min(6)])
            .split(area);
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);
        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(rows[1]);

        let color = self.status_color();
        let reason = self.warp_info.reason.as_deref().unwrap_or("—");
        let force = if self.force_disconnect {
            format!("Force disconnect: ON ({})", self.force_disconnect_kills)
        } else {
            "Force disconnect: off".into()
        };
        let connection = vec![
            Line::from(Span::styled(
                self.warp_info.status.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("Reason: {reason}")),
            Line::from(self.auto_connect_line()),
            Line::from(force),
        ];
        frame.render_widget(
            Paragraph::new(connection).block(
                Block::bordered()
                    .title("Connection")
                    .border_style(Style::default().fg(color)),
            ),
            top[0],
        );

        let org = self.warp_info.organization.as_deref().unwrap_or("—");
        let account = self.warp_info.account_type.as_deref().unwrap_or("—");
        let mode = self
            .warp_info
            .mode
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "—".into());
        let vnet = match (
            self.warp_info.vnet_name.as_deref(),
            self.warp_info.vnet_description.as_deref(),
        ) {
            (Some(name), Some(desc)) => format!("{name} ({desc})"),
            (Some(name), None) => name.to_string(),
            _ => "—".into(),
        };
        let identity = vec![
            Line::from(format!("Org: {org}")),
            Line::from(format!("Account: {account}")),
            Line::from(format!("Mode: {mode}")),
            Line::from(format!("VNet: {vnet}")),
            Line::from(format!(
                "Locked: {}",
                if self.warp_info.switch_locked {
                    "yes"
                } else {
                    "no"
                }
            )),
        ];
        frame.render_widget(
            Paragraph::new(identity).block(Block::bordered().title("Identity")),
            top[1],
        );

        let split_title = match self.warp_info.split_tunnel_mode.as_deref() {
            Some(mode) => format!("Split tunnels ({mode})"),
            None => "Split tunnels".into(),
        };
        let split_items: Vec<String> = self
            .warp_info
            .split_tunnels
            .iter()
            .map(|e| e.display_label())
            .collect();
        frame.render_widget(
            Paragraph::new(preview_lines(&split_items, 12))
                .block(Block::bordered().title(split_title))
                .wrap(Wrap { trim: true }),
            bottom[0],
        );

        frame.render_widget(
            Paragraph::new(preview_lines(&self.warp_info.fallback_domains, 12))
                .block(Block::bordered().title("DNS fallback"))
                .wrap(Wrap { trim: true }),
            bottom[1],
        );
    }

    fn render_details(&self, frame: &mut Frame, area: Rect) {
        let info = &self.warp_info;
        let mut lines = vec![
            Line::from(format!(
                "Organization: {}",
                info.organization.as_deref().unwrap_or("—")
            )),
            Line::from(format!(
                "Account type: {}",
                info.account_type.as_deref().unwrap_or("—")
            )),
            Line::from(format!(
                "Account ID: {}",
                info.account_id.as_deref().unwrap_or("—")
            )),
            Line::from(format!(
                "Device ID: {}",
                info.device_id.as_deref().unwrap_or("—")
            )),
            Line::from(format!(
                "Public key: {}",
                info.public_key.as_deref().unwrap_or("—")
            )),
            Line::from(format!(
                "VNet: {}",
                info.vnet_name.as_deref().unwrap_or("—")
            )),
            Line::from(format!(
                "Mode: {}",
                info.mode
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "—".into())
            )),
            Line::from(format!(
                "Always on: {}   Switch locked: {}",
                info.always_on, info.switch_locked
            )),
            Line::from(""),
            Line::from("DNS fallback:"),
        ];
        if info.fallback_domains.is_empty() {
            lines.push(Line::from("  none"));
        } else {
            lines.extend(
                info.fallback_domains
                    .iter()
                    .map(|d| Line::from(format!("  {d}"))),
            );
        }

        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::bordered().title("Details"))
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Paste(text) if self.input.is_some() => self.input_buf.push_str(&text),
                Event::Mouse(_) | Event::Resize(_, _) | Event::Paste(_) => {}
                _ => {}
            }
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        if key.modifiers == KeyModifiers::CONTROL
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            self.quit();
            return;
        }

        if self.input.is_some() {
            match key.code {
                KeyCode::Esc => self.cancel_input(),
                KeyCode::Enter => self.submit_input(),
                KeyCode::Backspace => {
                    self.input_buf.pop();
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.input_buf.push(c);
                }
                _ => {}
            }
            return;
        }

        if self.mode_selection.is_some() {
            match key.code {
                KeyCode::Esc => self.mode_selection = None,
                KeyCode::Up => self.select_previous(),
                KeyCode::Down => self.select_next(),
                KeyCode::Enter => self.handle_mode_select(),
                _ => {}
            }
            return;
        }

        if self.show_details {
            match key.code {
                KeyCode::Esc | KeyCode::Char('i') | KeyCode::Char('I') => {
                    self.show_details = false;
                }
                KeyCode::Char('y') | KeyCode::Char('Y') => self.copy_details(),
                KeyCode::Char('q') => self.quit(),
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') | KeyCode::Char('C') => self.handle_connect(),
            KeyCode::Char('p') | KeyCode::Char('P') => self.handle_pause(),
            KeyCode::Char('d') | KeyCode::Char('D') => self.handle_disconnect(),
            KeyCode::Char('a') | KeyCode::Char('A') => self.start_auth_input(),
            KeyCode::Char('t') | KeyCode::Char('T') => self.start_token_input(),
            KeyCode::Char('u') | KeyCode::Char('U') => self.start_unenroll_confirm(),
            KeyCode::Char('i') | KeyCode::Char('I') => self.show_details = true,
            KeyCode::Char('r') | KeyCode::Char('R') => self.update_warp_status(),
            KeyCode::Char('m') | KeyCode::Char('M') => self.handle_mode_selection(),
            _ => {}
        }
    }
}

fn preview_lines(items: &[String], max: usize) -> Vec<Line<'static>> {
    if items.is_empty() {
        return vec![Line::from("none")];
    }
    let mut lines: Vec<Line> = items
        .iter()
        .take(max)
        .map(|item| Line::from(item.clone()))
        .collect();
    if items.len() > max {
        lines.push(Line::from(format!("… {} more", items.len() - max)));
    }
    lines
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(windows)]
    let mut child = Command::new("clip")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    #[cfg(not(any(windows, target_os = "macos")))]
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .or_else(|_| {
            Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn()
        })
        .map_err(|e| e.to_string())?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("clipboard command exited {status}"))
    }
}
