//! Wingman: plays nxnative Switch games with a HOTAS (WinWing defaults). The HID reader (hid.rs) feeds the mapping engine
//! (engine.rs), which sends one Switch controller to the game 500 times a second; profiles (profile.rs) say how;
//! game.rs turns the game's external-controller input on (its env.txt) and launches it.
mod engine;
mod game;
mod hid;
mod profile;

use serde::Serialize;
use std::collections::BTreeMap;

const TITLE: &str = "0400E88025B1A000";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Live {
    devices: Vec<String>,
    inputs: BTreeMap<String, hid::DevState>,
    out: engine::Out,
    pressed: Vec<&'static str>,
    link: engine::Link,
    touched: Option<hid::Touched>,
    hid_error: Option<i32>,
}

#[tauri::command]
fn live() -> Live {
    let out = engine::out();
    let pressed = profile::BUTTONS.iter().enumerate().filter(|(i, _)| out.buttons & (1 << i) != 0).map(|(_, b)| *b).collect();
    Live { devices: hid::devices(), inputs: hid::snapshot(), out, pressed, link: engine::link(), touched: hid::touched(), hid_error: hid::open_error() }
}

#[tauri::command]
fn get_profile() -> profile::Profile { engine::profile() }

#[tauri::command]
fn save_profile(p: profile::Profile) -> Result<(), String> {
    profile::save(&p)?;
    let old_port = engine::profile().port;
    engine::set_profile(p.clone());
    if old_port != p.port && game::status(&p).enabled { game::set_enabled(&p, true)?; }
    Ok(())
}

#[tauri::command]
fn default_profile() -> profile::Profile {
    // The defaults, but keep where the game app is.
    profile::Profile { app_path: engine::profile().app_path, ..profile::star_fox() }
}

#[tauri::command]
fn game_status() -> game::GameStatus { game::status(&engine::profile()) }

#[tauri::command]
fn set_game_enabled(on: bool) -> Result<(), String> { game::set_enabled(&engine::profile(), on) }

#[tauri::command]
fn launch_game() -> Result<(), String> { engine::set_sending(true); game::launch(&engine::profile()) }

#[tauri::command]
fn set_sending(on: bool) { engine::set_sending(on); }

/// Lets the player choose the game app; returns the profile with the new path (unchanged when cancelled).
#[tauri::command]
fn pick_game_app() -> Result<profile::Profile, String> {
    let mut p = engine::profile();
    let Some(path) = game::pick_app()? else { return Ok(p) };
    match game::app_title_id(std::path::Path::new(&path)) {
        Some(id) if id == p.title_id => {}
        Some(id) => return Err(format!("that app is another game (title {id}); this profile is for {} ({})", p.name, p.title_id)),
        None => return Err("that app was not built by nxnative".into()),
    }
    p.app_path = path;
    profile::save(&p)?;
    engine::set_profile(p.clone());
    Ok(p)
}

pub fn run() {
    // Never keep a working directory on the removable volume (macOS would ask for access to it).
    let _ = std::env::set_current_dir("/");
    let mut p = profile::load(TITLE);
    if !std::path::Path::new(&p.app_path).is_dir() {
        // A fresh profile, or the app moved: look in the Applications folders (an unplugged drive keeps its path).
        if let Some(found) = game::find_app(TITLE) { p.app_path = found; let _ = profile::save(&p); }
    }
    engine::set_profile(p);
    tauri::Builder::default()
        .setup(|_app| {
            hid::start();
            engine::start();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![live, get_profile, save_profile, default_profile, game_status, set_game_enabled, launch_game, set_sending, pick_game_app])
        .build(tauri::generate_context!())
        .expect("error while running Wingman")
        .run(|_app, event| if let tauri::RunEvent::Exit = event { engine::leave(); });
}
