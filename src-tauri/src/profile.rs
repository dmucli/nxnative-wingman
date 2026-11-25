//! A game profile: how the WinWing controls become one Switch controller. Stored as JSON, one file per game, in
//! ~/Library/Application Support/Wingman/profiles/<title id>.json. The UI edits the same structure.
//!
//! Sources name a physical control by device role ("stick", "throttle", "pedals") and the HID usage it reports:
//! a button number (HID button page usage, the same numbers as DirectInput's JOY_BTN<n>),
//! a hat direction, or an axis name (x y z rx ry rz slider dial).
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ButtonSrc {
    pub dev: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button: Option<u32>,
    /// "up" "right" "down" "left": a diagonal counts for both of its directions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hat: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AxisSrc {
    pub dev: String,
    pub axis: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    /// Switch buttons held during this step (none = everything released).
    #[serde(default)]
    pub buttons: Vec<String>,
    pub ms: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    /// Pressed while the control is held.
    Hold,
    /// Pressed and released repeatedly while held (autofire). A Hold of the same button wins over it; when that Hold
    /// ends the turbo restarts with a release, so a charged shot is let go.
    Turbo,
    /// One short press when the control goes down.
    Tap,
    /// A timed sequence when the control goes down (e.g. a double-tap from one press).
    Macro,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Bind {
    pub src: ButtonSrc,
    pub mode: Mode,
    /// The Switch buttons for Hold / Turbo / Tap.
    #[serde(default)]
    pub buttons: Vec<String>,
    /// The sequence for Macro.
    #[serde(default)]
    pub steps: Vec<Step>,
    /// What it does in the game, shown in the UI.
    #[serde(default)]
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AxisMap {
    pub src: AxisSrc,
    /// "LX" "LY" "RX" "RY"
    pub to: String,
    #[serde(default)]
    pub invert: bool,
    /// Fraction of the half-range around the centre that reads as zero.
    #[serde(default = "default_deadzone")]
    pub deadzone: f32,
    /// 0 = linear, 1 = fully cubic (finer near the centre).
    #[serde(default)]
    pub expo: f32,
    /// Output multiplier (1 = full deflection at full travel).
    #[serde(default = "one")]
    pub scale: f32,
    #[serde(default)]
    pub label: String,
}
fn default_deadzone() -> f32 { 0.03 }
fn one() -> f32 { 1.0 }

/// A Switch button held while an axis (after invert, 0..1) is inside [from, to]: the throttle's boost and brake.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub src: AxisSrc,
    #[serde(default)]
    pub invert: bool,
    pub from: f32,
    pub to: f32,
    pub button: String,
    #[serde(default)]
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub title_id: String,
    /// The nxnative Mac app to launch.
    #[serde(default)]
    pub app_path: String,
    #[serde(default = "default_port")]
    pub port: u16,
    /// Autofire rate (presses a second) and the pressed fraction of each cycle.
    #[serde(default = "default_turbo_hz")]
    pub turbo_hz: f32,
    #[serde(default = "default_duty")]
    pub turbo_duty: f32,
    /// How long a Tap holds its buttons.
    #[serde(default = "default_tap_ms")]
    pub tap_ms: u32,
    #[serde(default)]
    pub axes: Vec<AxisMap>,
    #[serde(default)]
    pub zones: Vec<Zone>,
    #[serde(default)]
    pub binds: Vec<Bind>,
}
fn default_port() -> u16 { 47821 }
fn default_turbo_hz() -> f32 { 12.0 }
fn default_duty() -> f32 { 0.5 }
fn default_tap_ms() -> u32 { 60 }

/// Switch button names, in dm_input.h's (NpadButton) bit order.
pub const BUTTONS: [&str; 16] = ["A", "B", "X", "Y", "LS", "RS", "L", "R", "ZL", "ZR", "Plus", "Minus", "Left", "Up", "Right", "Down"];
pub fn button_bit(name: &str) -> u64 {
    BUTTONS.iter().position(|b| b.eq_ignore_ascii_case(name)).map_or(0, |i| 1u64 << i)
}
pub fn mask(names: &[String]) -> u64 { names.iter().fold(0, |m, n| m | button_bit(n)) }

fn b(dev: &str, n: u32) -> ButtonSrc { ButtonSrc { dev: dev.into(), button: Some(n), hat: None } }
fn hat(dev: &str, d: &str) -> ButtonSrc { ButtonSrc { dev: dev.into(), button: None, hat: Some(d.into()) } }
fn ax(dev: &str, a: &str) -> AxisSrc { AxisSrc { dev: dev.into(), axis: a.into() } }
fn bind(src: ButtonSrc, mode: Mode, buttons: &[&str], label: &str) -> Bind {
    Bind { src, mode, buttons: buttons.iter().map(|s| s.to_string()).collect(), steps: vec![], label: label.into() }
}
/// One press → two quick taps of `button` (Star Fox's barrel roll is a double-tap of L or R).
pub fn double_tap(button: &str) -> Vec<Step> {
    vec![
        Step { buttons: vec![button.into()], ms: 50 },
        Step { buttons: vec![], ms: 50 },
        Step { buttons: vec![button.into()], ms: 50 },
    ]
}

/// Star Fox (Switch 2) on the WinWing F/A-18 grip + throttle. Game buttons from game8's control list (A fire, hold
/// to charge; Y bomb; ZR boost; ZL brake; double-tap L/R barrel roll); button numbers are WinWing's HID usages.
pub fn star_fox() -> Profile {
    use Mode::*;
    Profile {
        name: "Star Fox".into(),
        title_id: "0400E88025B1A000".into(),
        app_path: String::new(), // found at startup (game::find_app) or chosen in the app
        port: default_port(),
        turbo_hz: 12.0,
        turbo_duty: 0.5,
        tap_ms: 60,
        axes: vec![
            AxisMap { src: ax("stick", "x"), to: "LX".into(), invert: false, deadzone: 0.03, expo: 0.25, scale: 1.0, label: "Steer left / right".into() },
            AxisMap { src: ax("stick", "y"), to: "LY".into(), invert: true, deadzone: 0.03, expo: 0.25, scale: 1.0, label: "Climb / dive".into() },
            AxisMap { src: ax("throttle", "x"), to: "RX".into(), invert: false, deadzone: 0.08, expo: 0.0, scale: 1.0, label: "TDC: camera / aim".into() },
            AxisMap { src: ax("throttle", "y"), to: "RY".into(), invert: true, deadzone: 0.08, expo: 0.0, scale: 1.0, label: "TDC: camera / aim".into() },
        ],
        zones: vec![
            Zone { src: ax("throttle", "ry"), invert: false, from: 0.0, to: 0.15, button: "ZL".into(), label: "Brake (throttle back)".into() },
            Zone { src: ax("throttle", "ry"), invert: false, from: 0.85, to: 1.0, button: "ZR".into(), label: "Boost (throttle forward)".into() },
        ],
        binds: vec![
            bind(b("stick", 9), Turbo, &["A"], "Trigger, 1st detent: rapid fire"),
            bind(b("stick", 10), Hold, &["A"], "Trigger, 2nd detent: charge, fire on release"),
            bind(b("stick", 7), Tap, &["Y"], "Pickle: bomb"),
            Bind { src: b("throttle", 14), mode: Macro, buttons: vec![], steps: double_tap("L"), label: "COMM left: barrel roll left".into() },
            Bind { src: b("throttle", 16), mode: Macro, buttons: vec![], steps: double_tap("R"), label: "COMM right: barrel roll right".into() },
            bind(b("stick", 17), Hold, &["B"], "Paddle: B (back in menus)"),
            bind(b("throttle", 21), Tap, &["Plus"], "Cage: pause (+)"),
            bind(b("throttle", 22), Hold, &["RS"], "TDC press: R3"),
            bind(hat("stick", "up"), Hold, &["Up"], "Trim hat: D-pad up"),
            bind(hat("stick", "down"), Hold, &["Down"], "Trim hat: D-pad down"),
            bind(hat("stick", "left"), Hold, &["Left"], "Trim hat: D-pad left"),
            bind(hat("stick", "right"), Hold, &["Right"], "Trim hat: D-pad right"),
            bind(b("stick", 1), Hold, &["A"], "Trim hat press: A (menus)"),
        ],
    }
}

pub fn dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join("Library/Application Support/Wingman/profiles")
}

fn valid_id(id: &str) -> bool { !id.is_empty() && id.len() <= 32 && id.chars().all(|c| c.is_ascii_alphanumeric()) }

pub fn load(id: &str) -> Profile {
    if valid_id(id) {
        if let Ok(s) = std::fs::read_to_string(dir().join(format!("{id}.json"))) {
            match serde_json::from_str::<Profile>(&s) {
                Ok(p) => return p,
                Err(e) => eprintln!("profile {id}: unreadable ({e}), using the defaults"),
            }
        }
    }
    star_fox()
}

pub fn save(p: &Profile) -> Result<(), String> {
    if !valid_id(&p.title_id) { return Err(format!("bad title id {:?}", p.title_id)); }
    std::fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    let path = dir().join(format!("{}.json", p.title_id));
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(p).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bits_follow_npad_order() {
        assert_eq!(button_bit("A"), 1);
        assert_eq!(button_bit("zr"), 1 << 9);
        assert_eq!(button_bit("Down"), 1 << 15);
        assert_eq!(button_bit("nope"), 0);
    }
    #[test]
    fn default_round_trips() {
        let p = star_fox();
        let s = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Profile>(&s).unwrap(), p);
    }
}
