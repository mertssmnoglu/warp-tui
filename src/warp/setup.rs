use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub fn default_cli_name() -> &'static str {
    if cfg!(windows) {
        "warp-cli.exe"
    } else {
        "warp-cli"
    }
}

pub fn resolve_warp_cli() -> Option<PathBuf> {
    let name = default_cli_name();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    known_paths().into_iter().find(|p| p.is_file())
}

fn known_paths() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        vec![PathBuf::from(
            r"C:\Program Files\Cloudflare\Cloudflare WARP\warp-cli.exe",
        )]
    }
    #[cfg(target_os = "macos")]
    {
        vec![
            PathBuf::from("/usr/local/bin/warp-cli"),
            PathBuf::from("/opt/homebrew/bin/warp-cli"),
            PathBuf::from("/Applications/Cloudflare WARP.app/Contents/Resources/warp-cli"),
        ]
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        vec![
            PathBuf::from("/usr/bin/warp-cli"),
            PathBuf::from("/bin/warp-cli"),
            PathBuf::from("/usr/local/bin/warp-cli"),
        ]
    }
}

/// Find warp-cli, or ask to install it. Best-effort PATH update on Windows.
pub fn ensure_warp_cli() -> Result<PathBuf, String> {
    if let Some(binary) = resolve_warp_cli() {
        try_add_to_path(&binary);
        return Ok(binary);
    }

    println!("Cloudflare WARP is not installed (warp-cli was not found).");
    print!("Install Cloudflare WARP now? [Y/n] ");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|e| e.to_string())?;
    let answer = answer.trim();
    if !answer.is_empty()
        && !answer.eq_ignore_ascii_case("y")
        && !answer.eq_ignore_ascii_case("yes")
    {
        return Err("Cloudflare WARP is required to run warp-tui".into());
    }

    install_warp()?;

    for _ in 0..20 {
        if let Some(binary) = resolve_warp_cli() {
            try_add_to_path(&binary);
            return Ok(binary);
        }
        thread::sleep(Duration::from_millis(500));
    }

    Err("WARP install finished but warp-cli was still not found".into())
}

fn install_warp() -> Result<(), String> {
    #[cfg(windows)]
    {
        println!("Installing Cloudflare WARP with winget (UAC may prompt)...");
        let status = Command::new("winget")
            .args([
                "install",
                "--id",
                "Cloudflare.WARP",
                "-e",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ])
            .status()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    "winget was not found. Install WARP from https://one.one.one.one/".to_string()
                } else {
                    e.to_string()
                }
            })?;
        if !status.success() {
            return Err(format!("winget install failed with status {status}"));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err(
            "Automatic install is currently Windows-only. Install WARP from https://developers.cloudflare.com/warp-client/"
                .into(),
        )
    }
}

fn try_add_to_path(binary: &Path) {
    let Some(dir) = binary.parent() else {
        return;
    };
    prepend_process_path(dir);
    #[cfg(windows)]
    persist_user_path(dir);
}

fn prepend_process_path(dir: &Path) {
    let Ok(path) = std::env::var("PATH") else {
        return;
    };
    if std::env::split_paths(&path).any(|p| p == dir) {
        return;
    }
    let mut dirs = vec![dir.to_path_buf()];
    dirs.extend(std::env::split_paths(&path));
    if let Ok(joined) = std::env::join_paths(dirs) {
        // SAFETY: called once from main before any worker threads.
        unsafe { std::env::set_var("PATH", joined) };
    }
}

#[cfg(windows)]
fn persist_user_path(dir: &Path) {
    let dir = dir.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$dir = '{dir}'; $p = [Environment]::GetEnvironmentVariable('Path','User'); if ($null -eq $p) {{ $p = '' }}; if ($p -notlike ('*' + $dir + '*')) {{ [Environment]::SetEnvironmentVariable('Path', ($p.TrimEnd(';') + ';' + $dir), 'User') }}"
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status();
}
