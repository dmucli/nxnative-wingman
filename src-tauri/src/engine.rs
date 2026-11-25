//! The mapping engine: every tick (2 ms) the HOTAS state and the profile make one Switch controller state, which is
//! sent to the game over localhost UDP (nxnative's DM_EXT_PAD, dm/host/ext_pad_mac.h in the nxnative checkout).
//!
//! Button rules, per Switch button:
//!   * Hold binds, throttle zones, running taps and macros press it ("held").
//!   * Turbo binds press and release it at the profile's rate, but only while nothing holds it. A turbo that starts
//!     right after a hold ended starts with a release: the trigger's second detent holds A (a charge), and easing back
//!     to the first detent lets A go (the charged shot) before the rapid fire resumes.
use crate::hid::{self, DevState};
use crate::profile::{self, Mode, Profile};
use serde::Serialize;
use std::collections::BTreeMap;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const STICK_MAX: f32 = 32767.0;
const ZONE_HYST: f32 = 0.02;
const MIN_PHASE_MS: f64 = 20.0;   // a turbo press or release shorter than ~1 frame at 60 fps could be missed

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Out {
    pub buttons: u64,
    pub axes: [i32; 4],   // LX LY RX RY
}

#[derive(Default)]
pub struct Engine {
    prev_held: u64,
    turbo_eff: u64,
    turbo_t0: [f64; 16],
    prev_active: Vec<bool>,
    taps: Vec<(u64, f64)>,          // mask, until
    macros: Vec<(usize, f64)>,      // bind index, start
    zone_on: Vec<bool>,
}

fn dev<'a>(inp: &'a BTreeMap<String, DevState>, d: &str) -> Option<&'a DevState> { inp.get(d) }

pub fn src_active(inp: &BTreeMap<String, DevState>, s: &profile::ButtonSrc) -> bool {
    let Some(ds) = dev(inp, &s.dev) else { return false };
    if let Some(n) = s.button { return ds.buttons.contains(&n); }
    if let Some(h) = &s.hat { return hid::hat_dir(h).is_some_and(|vals| vals.contains(&ds.hat)); }
    false
}

pub fn axis_value(inp: &BTreeMap<String, DevState>, s: &profile::AxisSrc) -> Option<f32> {
    dev(inp, &s.dev).and_then(|ds| ds.axes.get(&s.axis).copied())
}

/// A centred axis (0..1, rest at 0.5) → -1..1 after invert, deadzone, expo and scale.
pub fn shape(v: f32, m: &profile::AxisMap) -> f32 {
    let mut c = v * 2.0 - 1.0;
    if m.invert { c = -c; }
    let dz = m.deadzone.clamp(0.0, 0.9);
    let a = c.abs();
    c = if a <= dz { 0.0 } else { c.signum() * (a - dz) / (1.0 - dz) };
    let e = m.expo.clamp(0.0, 1.0);
    c = (1.0 - e) * c + e * c * c * c;
    (c * m.scale).clamp(-1.0, 1.0)
}

fn axis_index(to: &str) -> Option<usize> {
    match to.to_ascii_uppercase().as_str() { "LX" => Some(0), "LY" => Some(1), "RX" => Some(2), "RY" => Some(3), _ => None }
}

impl Engine {
    pub fn tick(&mut self, p: &Profile, inp: &BTreeMap<String, DevState>, now: f64) -> Out {
        if self.prev_active.len() != p.binds.len() { self.prev_active = vec![false; p.binds.len()]; self.macros.clear(); }
        if self.zone_on.len() != p.zones.len() { self.zone_on = vec![false; p.zones.len()]; }

        let mut held = 0u64;
        let mut turbo_want = 0u64;
        for (i, b) in p.binds.iter().enumerate() {
            let active = src_active(inp, &b.src);
            let pressed_now = active && !self.prev_active[i];
            self.prev_active[i] = active;
            let m = profile::mask(&b.buttons);
            match b.mode {
                Mode::Hold => if active { held |= m },
                Mode::Turbo => if active { turbo_want |= m },
                Mode::Tap => if pressed_now { self.taps.push((m, now + p.tap_ms.max(17) as f64)) },
                Mode::Macro => if pressed_now && !self.macros.iter().any(|(j, _)| *j == i) && !b.steps.is_empty() { self.macros.push((i, now)) },
            }
        }

        for (i, z) in p.zones.iter().enumerate() {
            let Some(mut v) = axis_value(inp, &z.src) else { self.zone_on[i] = false; continue };
            if z.invert { v = 1.0 - v; }
            let (lo, hi) = if self.zone_on[i] { (z.from - ZONE_HYST, z.to + ZONE_HYST) } else { (z.from, z.to) };
            self.zone_on[i] = v >= lo && v <= hi;
            if self.zone_on[i] { held |= profile::button_bit(&z.button); }
        }

        self.taps.retain(|(_, until)| *until > now);
        for (m, _) in &self.taps { held |= m; }

        let binds = &p.binds;
        self.macros.retain(|(i, start)| {
            let Some(b) = binds.get(*i) else { return false };
            let mut t = now - start;
            for s in &b.steps {
                if t < s.ms as f64 { held |= profile::mask(&s.buttons); return true; }
                t -= s.ms as f64;
            }
            false
        });

        let period = 1000.0 / (p.turbo_hz.clamp(1.0, 30.0) as f64);
        let on_ms = (period * p.turbo_duty.clamp(0.1, 0.9) as f64).clamp(MIN_PHASE_MS.min(period / 2.0), (period - MIN_PHASE_MS).max(period / 2.0));
        let eff = turbo_want & !held;
        let mut turbo_on = 0u64;
        for bit in 0..16 {
            let m = 1u64 << bit;
            if eff & m == 0 { continue; }
            if self.turbo_eff & m == 0 {
                // a fresh turbo: start pressed, unless this button was held a moment ago (start released, so the held press ends)
                self.turbo_t0[bit] = if self.prev_held & m != 0 { now - on_ms } else { now };
            }
            if (now - self.turbo_t0[bit]).rem_euclid(period) < on_ms { turbo_on |= m; }
        }
        self.turbo_eff = eff;
        self.prev_held = held;

        let mut axes = [0f32; 4];
        for a in &p.axes {
            let (Some(i), Some(v)) = (axis_index(&a.to), axis_value(inp, &a.src)) else { continue };
            let c = shape(v, a);
            if c.abs() > axes[i].abs() { axes[i] = c; }
        }
        Out { buttons: held | turbo_on, axes: axes.map(|c| (c * STICK_MAX).round() as i32) }
    }
}

pub fn packet(seq: u32, out: &Out, leave: bool) -> [u8; 36] {
    let mut b = [0u8; 36];
    b[0..4].copy_from_slice(b"NXEP");
    b[4..6].copy_from_slice(&1u16.to_le_bytes());
    b[6..8].copy_from_slice(&(leave as u16).to_le_bytes());
    b[8..12].copy_from_slice(&seq.to_le_bytes());
    b[12..20].copy_from_slice(&out.buttons.to_le_bytes());
    for (i, a) in out.axes.iter().enumerate() { b[20 + 4 * i..24 + 4 * i].copy_from_slice(&a.to_le_bytes()); }
    b
}

// ---- runtime ----------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    /// The player the game gave us (1-based), 0 when it has not answered.
    pub player: u16,
    /// Milliseconds since its last answer (it answers about once a second), or -1.
    pub ack_age_ms: i64,
    pub sending: bool,
    pub error: Option<String>,
}

struct Shared {
    profile: Arc<Profile>,
    out: Out,
    link: Link,
    last_ack: Option<Instant>,
}

fn shared() -> &'static Mutex<Shared> {
    static S: OnceLock<Mutex<Shared>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Shared { profile: Arc::new(profile::star_fox()), out: Out::default(), link: Link::default(), last_ack: None }))
}
static SENDING: AtomicBool = AtomicBool::new(true);

pub fn set_profile(p: Profile) { shared().lock().unwrap().profile = Arc::new(p); }
pub fn profile() -> Profile { (*shared().lock().unwrap().profile).clone() }
pub fn out() -> Out { shared().lock().unwrap().out }
pub fn link() -> Link {
    let s = shared().lock().unwrap();
    let mut l = s.link.clone();
    l.ack_age_ms = s.last_ack.map_or(-1, |t| t.elapsed().as_millis() as i64);
    if l.ack_age_ms > 3000 || l.ack_age_ms < 0 { l.player = 0; }
    l.sending = SENDING.load(Ordering::Relaxed);
    l
}
pub fn set_sending(on: bool) { SENDING.store(on, Ordering::Relaxed); }

pub fn start() {
    std::thread::spawn(|| {
        let sock = match UdpSocket::bind("127.0.0.1:0") {
            Ok(s) => s,
            Err(e) => { shared().lock().unwrap().link.error = Some(format!("no UDP socket: {e}")); return; }
        };
        let _ = sock.set_nonblocking(true);
        let t0 = Instant::now();
        let mut eng = Engine::default();
        let mut seq: u32 = 0;
        let mut was_sending = true;
        let mut ack = [0u8; 16];
        loop {
            let p = shared().lock().unwrap().profile.clone();
            let addr = format!("127.0.0.1:{}", p.port);
            let inp = hid::snapshot();
            let now = t0.elapsed().as_secs_f64() * 1000.0;
            let o = eng.tick(&p, &inp, now);
            let sending = SENDING.load(Ordering::Relaxed);
            if sending {
                seq = seq.wrapping_add(1);
                let _ = sock.send_to(&packet(seq, &o, false), &addr);
            } else if was_sending {
                seq = seq.wrapping_add(1);
                let _ = sock.send_to(&packet(seq, &Out::default(), true), &addr);   // free the game's slot
            }
            was_sending = sending;
            while let Ok((n, _)) = sock.recv_from(&mut ack) {
                if n >= 8 && &ack[0..4] == b"NXEA" {
                    let mut s = shared().lock().unwrap();
                    s.link.player = u16::from_le_bytes([ack[6], ack[7]]);
                    s.last_ack = Some(Instant::now());
                }
            }
            shared().lock().unwrap().out = if sending { o } else { Out::default() };
            std::thread::sleep(Duration::from_millis(2));
        }
    });
}

/// On quit: tell the game we are gone (it would release everything 300 ms later anyway).
pub fn leave() {
    if let Ok(sock) = UdpSocket::bind("127.0.0.1:0") {
        let port = shared().lock().unwrap().profile.port;
        let _ = sock.send_to(&packet(u32::MAX / 2, &Out::default(), true), format!("127.0.0.1:{port}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::star_fox;

    fn inp(stick: &[u32], throttle: &[u32], ry: f32) -> BTreeMap<String, DevState> {
        let mut m = BTreeMap::new();
        let mut s = DevState { hat: -1, ..Default::default() };
        s.buttons.extend(stick.iter().copied());
        s.axes.insert("x".into(), 0.5); s.axes.insert("y".into(), 0.5);
        let mut t = DevState { hat: -1, ..Default::default() };
        t.buttons.extend(throttle.iter().copied());
        t.axes.insert("ry".into(), ry); t.axes.insert("x".into(), 0.5); t.axes.insert("y".into(), 0.5);
        m.insert("stick".into(), s); m.insert("throttle".into(), t);
        m
    }
    const A: u64 = 1;

    #[test]
    fn first_detent_autofires() {
        let p = star_fox();
        let mut e = Engine::default();
        let i = inp(&[9], &[], 0.5);
        let seq: Vec<bool> = (0..250).map(|t| e.tick(&p, &i, t as f64 * 2.0).buttons & A != 0).collect();
        let presses = seq.windows(2).filter(|w| !w[0] && w[1]).count() + seq[0] as usize;
        assert!(seq[0], "starts pressed");
        assert!((5..=7).contains(&presses), "about 12 presses a second over 0.5 s, got {presses}");
    }

    #[test]
    fn second_detent_holds_then_releases_for_the_charged_shot() {
        let p = star_fox();
        let mut e = Engine::default();
        for t in 0..50 { e.tick(&p, &inp(&[9], &[], 0.5), t as f64 * 2.0); }
        for t in 50..600 { assert_ne!(e.tick(&p, &inp(&[9, 10], &[], 0.5), t as f64 * 2.0).buttons & A, 0, "held steadily while charging"); }
        // back to the first detent: A must go up at once (the shot) and stay up for at least a frame
        let after: Vec<bool> = (600..620).map(|t| e.tick(&p, &inp(&[9], &[], 0.5), t as f64 * 2.0).buttons & A != 0).collect();
        assert!(!after[0], "released at once");
        assert!(after.iter().take(10).all(|b| !b), "released for >= 20 ms");
    }

    #[test]
    fn one_press_is_a_double_tap() {
        let p = star_fox();
        let mut e = Engine::default();
        let l = 1u64 << 6;
        let mut seq = vec![];
        for t in 0..150 {
            let held = if t < 5 { &[14u32][..] } else { &[][..] };   // a 10 ms press of COMM left
            seq.push(e.tick(&p, &inp(&[], held, 0.5), t as f64 * 2.0).buttons & l != 0);
        }
        let presses = seq.windows(2).filter(|w| !w[0] && w[1]).count() + seq[0] as usize;
        assert_eq!(presses, 2);
        assert!(!seq[149], "done after 150 ms");
    }

    #[test]
    fn throttle_zones_with_hysteresis() {
        let p = star_fox();
        let mut e = Engine::default();
        let (zl, zr) = (1u64 << 8, 1u64 << 9);
        assert_eq!(e.tick(&p, &inp(&[], &[], 0.5), 0.0).buttons & (zl | zr), 0);
        assert_ne!(e.tick(&p, &inp(&[], &[], 0.9), 2.0).buttons & zr, 0);
        assert_ne!(e.tick(&p, &inp(&[], &[], 0.84), 4.0).buttons & zr, 0, "hysteresis keeps it");
        assert_eq!(e.tick(&p, &inp(&[], &[], 0.8), 6.0).buttons & zr, 0);
        assert_ne!(e.tick(&p, &inp(&[], &[], 0.05), 8.0).buttons & zl, 0);
    }

    #[test]
    fn stick_shaping() {
        let p = star_fox();
        let y = &p.axes[1];
        assert_eq!(shape(0.5, y), 0.0);
        assert!((shape(0.0, y) - 1.0).abs() < 1e-6, "stick forward = Switch stick up (inverted Y)");
        assert!(shape(0.51, y).abs() < 1e-6, "inside the deadzone");
    }

    #[test]
    fn packet_layout() {
        let b = packet(7, &Out { buttons: 0x201, axes: [1, -1, 32767, -32767] }, false);
        assert_eq!(&b[0..4], b"NXEP");
        assert_eq!(u32::from_le_bytes(b[8..12].try_into().unwrap()), 7);
        assert_eq!(u64::from_le_bytes(b[12..20].try_into().unwrap()), 0x201);
        assert_eq!(i32::from_le_bytes(b[24..28].try_into().unwrap()), -1);
    }
}
