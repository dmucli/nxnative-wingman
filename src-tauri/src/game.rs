//! The nxnative side: the game's env.txt (~/Library/Application Support/nxnative/<title id>/env.txt), where
//! DM_EXT_PAD=<port> turns on the external controller and DM_HID_PADS=0 stops the game from also reading the
//! HOTAS itself as raw HID gamepads (they would become extra players). Other lines are kept as they are.
use crate::profile::Profile;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

pub fn env_path(title_id: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join("Library/Application Support/nxnative").join(title_id).join("env.txt")
}

/// The env.txt text with our keys set (or removed when `on` is false), every other line unchanged.
pub fn edit_env(text: &str, port: u16, on: bool) -> String {
    let ours = |l: &str| { let k = l.trim_start().split('=').next().unwrap_or("").trim(); k == "DM_EXT_PAD" || k == "DM_HID_PADS" };
    let mut lines: Vec<String> = text.lines().filter(|l| !ours(l)).map(|l| l.to_string()).collect();
    while lines.last().is_some_and(|l| l.trim().is_empty()) { lines.pop(); }
    if on {
        lines.push(format!("DM_EXT_PAD={port}"));
        lines.push("DM_HID_PADS=0".into());
    }
    let mut s = lines.join("\n");
    if !s.is_empty() { s.push('\n'); }
    s
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStatus {
    pub env_path: String,
    pub enabled: bool,
    pub app_exists: bool,
    pub running: bool,
}

fn exe_dir(app: &str) -> String { format!("{}/Contents/MacOS/", app.trim_end_matches('/')) }

pub fn running(p: &Profile) -> bool {
    if p.app_path.is_empty() { return false; }
    Command::new("/usr/bin/pgrep").arg("-f").arg(exe_dir(&p.app_path)).output().map(|o| o.status.success()).unwrap_or(false)
}

pub fn status(p: &Profile) -> GameStatus {
    let path = env_path(&p.title_id);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let want = format!("DM_EXT_PAD={}", p.port);
    let enabled = text.lines().any(|l| l.trim() == want) && text.lines().any(|l| l.trim() == "DM_HID_PADS=0");
    GameStatus {
        env_path: path.display().to_string(),
        enabled,
        app_exists: !p.app_path.is_empty() && std::path::Path::new(&p.app_path).is_dir(),
        running: running(p),
    }
}

pub fn set_enabled(p: &Profile, on: bool) -> Result<(), String> {
    let path = env_path(&p.title_id);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let new = edit_env(&text, p.port, on);
    if new == text { return Ok(()); }
    if let Some(d) = path.parent() { std::fs::create_dir_all(d).map_err(|e| e.to_string())?; }
    std::fs::write(&path, new).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn launch(p: &Profile) -> Result<(), String> {
    if !std::path::Path::new(&p.app_path).is_dir() { return Err(if p.app_path.is_empty() { "choose the game app first (Game → Choose…)".into() } else { format!("not found: {} (is its drive connected?)", p.app_path) }); }
    set_enabled(p, true)?;
    let ok = Command::new("/usr/bin/open").arg(&p.app_path).status().map_err(|e| e.to_string())?.success();
    if ok { Ok(()) } else { Err(format!("open {} failed", p.app_path)) }
}

/// The title id of an app nxnative built (bundle id "org.nxnative.<title id>"), read from its Info.plist. Works on
/// XML and binary plists alike: the bundle id is stored as plain ASCII in both.
pub fn app_title_id(app: &std::path::Path) -> Option<String> {
    let bytes = std::fs::read(app.join("Contents/Info.plist")).ok()?;
    let needle = b"org.nxnative.";
    let at = bytes.windows(needle.len()).position(|w| w.eq_ignore_ascii_case(needle))? + needle.len();
    let id: String = bytes[at..].iter().take(16).map(|&b| b as char).collect();
    (id.len() == 16 && id.chars().all(|c| c.is_ascii_hexdigit())).then(|| id.to_ascii_uppercase())
}

/// Looks for the game's app in /Applications and ~/Applications (and one folder level below them). External drives
/// are not searched: macOS would ask for access to them. Choose… covers apps kept there.
pub fn find_app(title_id: &str) -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let roots = [PathBuf::from("/Applications"), PathBuf::from(home).join("Applications")];
    let is_it = |p: &std::path::Path| p.extension().is_some_and(|e| e == "app") && app_title_id(p).as_deref() == Some(title_id);
    for root in roots {
        let Ok(rd) = std::fs::read_dir(&root) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if is_it(&p) { return Some(p.display().to_string()); }
            if p.is_dir() && p.extension().is_none() {
                if let Ok(sub) = std::fs::read_dir(&p) {
                    for s in sub.flatten() { if is_it(&s.path()) { return Some(s.path().display().to_string()); } }
                }
            }
        }
    }
    None
}

/// Shows the macOS "choose an app" dialog. Ok(None) when cancelled.
pub fn pick_app() -> Result<Option<String>, String> {
    let script = r#"POSIX path of (choose file of type {"com.apple.application-bundle"} with prompt "Choose the game app built by nxnative" default location (path to applications folder))"#;
    let o = Command::new("/usr/bin/osascript").arg("-e").arg(script).output().map_err(|e| e.to_string())?;
    if !o.status.success() { return Ok(None); } // cancelled
    let path = String::from_utf8_lossy(&o.stdout).trim().trim_end_matches('/').to_string();
    Ok((!path.is_empty()).then_some(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn env_keeps_other_lines() {
        let t = "DM_SHOW_FPS=1\nDM_EXT_PAD=1\n\n";
        assert_eq!(edit_env(t, 47821, true), "DM_SHOW_FPS=1\nDM_EXT_PAD=47821\nDM_HID_PADS=0\n");
        assert_eq!(edit_env("DM_SHOW_FPS=1\nDM_EXT_PAD=47821\nDM_HID_PADS=0\n", 47821, false), "DM_SHOW_FPS=1\n");
        assert_eq!(edit_env("", 47821, false), "");
    }
    #[test]
    fn title_id_from_bundle_id() {
        let app = std::env::temp_dir().join(format!("wingman-test-{}.app", std::process::id()));
        std::fs::create_dir_all(app.join("Contents")).unwrap();
        let plist = |id: &str| format!("<plist><dict><key>CFBundleIdentifier</key><string>{id}</string></dict></plist>");
        std::fs::write(app.join("Contents/Info.plist"), plist("org.nxnative.0400e88025b1a000")).unwrap();
        assert_eq!(app_title_id(&app).as_deref(), Some("0400E88025B1A000"));
        std::fs::write(app.join("Contents/Info.plist"), plist("com.example.other")).unwrap();
        assert_eq!(app_title_id(&app), None);
        std::fs::remove_dir_all(&app).unwrap();
    }
}
