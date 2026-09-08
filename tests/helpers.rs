use warp_tui::resolve_warp_cli;

pub fn is_warp_cli_available() -> bool {
    resolve_warp_cli().is_some()
}
