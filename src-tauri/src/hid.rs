//! Live HOTAS input through IOKit's HID manager: joysticks and gamepads, opened without seizing them, so other
//! apps (a flight sim, a binding tool) can read the same devices at the same time.
//!
//! Devices get a role from their product name: "throttle" (…THROTTLE…), "pedals" (…RUDDER… / …PEDAL…), "stick"
//! (…JOYSTICK… / …STICK… / …GRIP…), else the product name itself (bind those with Learn). The state of each is kept here; the engine reads a snapshot every tick.
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{kCFAllocatorDefault, CFAllocatorRef, CFType, CFTypeRef, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopRef, CFRunLoopRun};
use core_foundation::string::{CFString, CFStringRef};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ffi::c_void;
use std::sync::{Mutex, MutexGuard, OnceLock};

type Ref = *mut c_void;
type IOReturn = i32;
type ValueCallback = extern "C" fn(*mut c_void, IOReturn, *mut c_void, Ref);
type DeviceCallback = extern "C" fn(*mut c_void, IOReturn, *mut c_void, Ref);

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOHIDManagerCreate(allocator: CFAllocatorRef, options: u32) -> Ref;
    fn IOHIDManagerSetDeviceMatchingMultiple(manager: Ref, multiple: CFArrayRef);
    fn IOHIDManagerRegisterInputValueCallback(manager: Ref, callback: ValueCallback, context: *mut c_void);
    fn IOHIDManagerRegisterDeviceMatchingCallback(manager: Ref, callback: DeviceCallback, context: *mut c_void);
    fn IOHIDManagerRegisterDeviceRemovalCallback(manager: Ref, callback: DeviceCallback, context: *mut c_void);
    fn IOHIDManagerScheduleWithRunLoop(manager: Ref, run_loop: CFRunLoopRef, mode: CFStringRef);
    fn IOHIDManagerOpen(manager: Ref, options: u32) -> IOReturn;
    fn IOHIDValueGetElement(value: Ref) -> Ref;
    fn IOHIDValueGetIntegerValue(value: Ref) -> isize;
    fn IOHIDElementGetUsagePage(element: Ref) -> u32;
    fn IOHIDElementGetUsage(element: Ref) -> u32;
    fn IOHIDElementGetLogicalMin(element: Ref) -> isize;
    fn IOHIDElementGetLogicalMax(element: Ref) -> isize;
    fn IOHIDElementGetDevice(element: Ref) -> Ref;
    fn IOHIDDeviceGetProperty(device: Ref, key: CFStringRef) -> CFTypeRef;
}

/// Axis names by HID generic-desktop usage 0x30..0x37.
pub const AXES: [&str; 8] = ["x", "y", "z", "rx", "ry", "rz", "slider", "dial"];

#[derive(Clone, Debug, Default, Serialize)]
pub struct DevState {
    /// Buttons down (HID button usages).
    pub buttons: BTreeSet<u32>,
    /// Axis name → 0..1 (logical min → 0).
    pub axes: BTreeMap<String, f32>,
    /// Hat value 0..7 (0 = up, clockwise), or -1 centred.
    pub hat: i32,
}

/// The last control touched, for the UI's Learn button.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Touched {
    pub dev: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,
    pub seq: u64,
}

#[derive(Default)]
struct State {
    devices: HashMap<usize, String>,
    devs: BTreeMap<String, DevState>,
    /// Where each axis was when it was first seen, so Learn catches a real movement, not jitter.
    axis_rest: HashMap<(String, String), f32>,
    touched: Option<Touched>,
    seq: u64,
    open_error: Option<i32>,
}

fn lock() -> MutexGuard<'static, State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(State::default())).lock().unwrap_or_else(|e| e.into_inner())
}

pub fn snapshot() -> BTreeMap<String, DevState> { lock().devs.clone() }
pub fn touched() -> Option<Touched> { lock().touched.clone() }
pub fn devices() -> Vec<String> { let mut v: Vec<String> = lock().devices.values().cloned().collect(); v.sort(); v.dedup(); v }
pub fn open_error() -> Option<i32> { lock().open_error }

pub fn hat_dir(d: &str) -> Option<[i32; 3]> {
    // The hat values that count as a direction: the direction itself and its two diagonals.
    match d { "up" => Some([7, 0, 1]), "right" => Some([1, 2, 3]), "down" => Some([3, 4, 5]), "left" => Some([5, 6, 7]), _ => None }
}
fn hat_name(v: i32) -> Option<&'static str> {
    match v { 0 => Some("up"), 2 => Some("right"), 4 => Some("down"), 6 => Some("left"), _ => None }
}

fn property(device: Ref, key: &'static str) -> Option<CFType> {
    let key = CFString::from_static_string(key);
    let p = unsafe { IOHIDDeviceGetProperty(device, key.as_concrete_TypeRef()) };
    if p.is_null() { None } else { Some(unsafe { CFType::wrap_under_get_rule(p) }) }
}
fn product(device: Ref) -> String {
    property(device, "Product").and_then(|p| p.downcast::<CFString>()).map(|s| s.to_string()).unwrap_or_else(|| "?".into())
}
pub fn role(name: &str) -> String {
    let u = name.to_uppercase();
    // Throttle first: some throttles call themselves "… Joystick" too.
    if u.contains("THROTTLE") { "throttle".into() }
    else if u.contains("RUDDER") || u.contains("PEDAL") { "pedals".into() }
    else if u.contains("JOYSTICK") || u.contains("STICK") || u.contains("GRIP") { "stick".into() }
    else { name.to_string() }
}

extern "C" fn on_value(_ctx: *mut c_void, _r: IOReturn, _sender: *mut c_void, value: Ref) {
    if value.is_null() { return; }
    let (el, v) = unsafe { (IOHIDValueGetElement(value), IOHIDValueGetIntegerValue(value)) };
    if el.is_null() { return; }
    let (page, usage) = unsafe { (IOHIDElementGetUsagePage(el), IOHIDElementGetUsage(el)) };
    let device = unsafe { IOHIDElementGetDevice(el) };
    let mut st = lock();
    let dev = match st.devices.get(&(device as usize)) { Some(d) => d.clone(), None => return };
    if page == 9 {
        let ds = st.devs.entry(dev.clone()).or_default();
        let down = v != 0;
        let changed = if down { ds.buttons.insert(usage) } else { ds.buttons.remove(&usage) };
        if down && changed { st.seq += 1; let seq = st.seq; st.touched = Some(Touched { dev, button: Some(usage), hat: None, axis: None, seq }); }
    } else if page == 1 && usage == 0x39 {
        let (lo, hi) = unsafe { (IOHIDElementGetLogicalMin(el), IOHIDElementGetLogicalMax(el)) };
        let h = if (v as isize) >= lo && (v as isize) <= hi && hi - lo >= 7 { (v as isize - lo) as i32 } else { -1 };
        let ds = st.devs.entry(dev.clone()).or_default();
        let old = ds.hat; ds.hat = h;
        if h != old { if let Some(n) = hat_name(h) { st.seq += 1; let seq = st.seq; st.touched = Some(Touched { dev, button: None, hat: Some(n.into()), axis: None, seq }); } }
    } else if page == 1 && (0x30..=0x37).contains(&usage) {
        let (lo, hi) = unsafe { (IOHIDElementGetLogicalMin(el), IOHIDElementGetLogicalMax(el)) };
        let f = if hi > lo { ((v as f64 - lo as f64) / (hi as f64 - lo as f64)).clamp(0.0, 1.0) as f32 } else { 0.0 };
        let name = AXES[(usage - 0x30) as usize].to_string();
        st.devs.entry(dev.clone()).or_default().axes.insert(name.clone(), f);
        let key = (dev.clone(), name.clone());
        let rest = *st.axis_rest.entry(key).or_insert(f);
        if (f - rest).abs() > 0.35 {
            let fresh = !matches!(&st.touched, Some(t) if t.dev == dev && t.axis.as_deref() == Some(name.as_str()));
            if fresh { st.seq += 1; let seq = st.seq; st.touched = Some(Touched { dev: dev.clone(), button: None, hat: None, axis: Some(name.clone()), seq }); }
            st.axis_rest.insert((dev, name), f);
        }
    }
}

extern "C" fn on_match(_ctx: *mut c_void, _r: IOReturn, _sender: *mut c_void, device: Ref) {
    if device.is_null() { return; }
    let name = role(&product(device));
    let mut st = lock();
    st.devices.insert(device as usize, name.clone());
    st.devs.entry(name).or_default().hat = -1;
}

extern "C" fn on_remove(_ctx: *mut c_void, _r: IOReturn, _sender: *mut c_void, device: Ref) {
    let mut st = lock();
    if let Some(name) = st.devices.remove(&(device as usize)) {
        if !st.devices.values().any(|n| *n == name) { st.devs.remove(&name); }
    }
}

fn run_hid_manager() {
    unsafe {
        let manager = IOHIDManagerCreate(kCFAllocatorDefault, 0);
        if manager.is_null() { lock().open_error = Some(-1); return; }
        let dict = |usage: i32| {
            CFDictionary::from_CFType_pairs(&[
                (CFString::from_static_string("DeviceUsagePage"), CFNumber::from(1i32)),
                (CFString::from_static_string("DeviceUsage"), CFNumber::from(usage)),
            ])
        };
        let matching = CFArray::from_CFTypes(&[dict(4), dict(5)]); // joysticks and gamepads
        IOHIDManagerSetDeviceMatchingMultiple(manager, matching.as_concrete_TypeRef());
        IOHIDManagerRegisterDeviceMatchingCallback(manager, on_match, std::ptr::null_mut());
        IOHIDManagerRegisterDeviceRemovalCallback(manager, on_remove, std::ptr::null_mut());
        IOHIDManagerRegisterInputValueCallback(manager, on_value, std::ptr::null_mut());
        IOHIDManagerScheduleWithRunLoop(manager, CFRunLoop::get_current().as_concrete_TypeRef(), kCFRunLoopDefaultMode);
        let r = IOHIDManagerOpen(manager, 0);
        if r != 0 { lock().open_error = Some(r); }
        CFRunLoopRun();   // the manager lives as long as the app
    }
}

pub fn start() { std::thread::spawn(run_hid_manager); }

#[cfg(test)]
mod tests {
    #[test]
    fn roles_from_product_names() {
        use super::role;
        assert_eq!(role("WINWING ORION JOYSTICK BASE 2 + JGRIP-F18"), "stick");
        assert_eq!(role("WINWING ORION THROTTLE BASE II + F18 HANDLE"), "throttle");
        assert_eq!(role("WINWING ORION RUDDER"), "pedals");
        assert_eq!(role("T.16000M"), "T.16000M");
    }
}
