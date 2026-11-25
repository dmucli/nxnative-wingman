// WinWing control names (F/A-18 grip on the Orion 2 base, Orion 2 throttle with the F/A-18 handle, rudder pedals).
// Button numbers are HID button usages (= DirectInput JOY_BTN<n>). Other devices still work: Learn stores whatever
// they report, and the list shows it as "<device> button <n>".
export type Control = { id: string; dev: string; group: string; label: string; button?: number; hat?: string; pulse?: boolean };
export type AxisControl = { dev: string; axis: string; group: string; label: string };
export const CONTROLS: Control[] = [
 {
  "id": "S.trigger1",
  "dev": "stick",
  "group": "Stick",
  "label": "Trigger, first detent",
  "button": 9
 },
 {
  "id": "S.trigger2",
  "dev": "stick",
  "group": "Stick",
  "label": "Trigger, second detent (fully pulled; the first detent is also pressed)",
  "button": 10
 },
 {
  "id": "S.weapon_release",
  "dev": "stick",
  "group": "Stick",
  "label": "Red Weapon Release button (thumb, top)",
  "button": 7
 },
 {
  "id": "S.sensor_fwd",
  "dev": "stick",
  "group": "Stick",
  "label": "Sensor Control (castle): fwd",
  "button": 3
 },
 {
  "id": "S.sensor_aft",
  "dev": "stick",
  "group": "Stick",
  "label": "Sensor Control: aft",
  "button": 5
 },
 {
  "id": "S.sensor_left",
  "dev": "stick",
  "group": "Stick",
  "label": "Sensor Control: left",
  "button": 6
 },
 {
  "id": "S.sensor_right",
  "dev": "stick",
  "group": "Stick",
  "label": "Sensor Control: right",
  "button": 4
 },
 {
  "id": "S.sensor_press",
  "dev": "stick",
  "group": "Stick",
  "label": "Sensor Control: press",
  "button": 2
 },
 {
  "id": "S.wpn_fwd",
  "dev": "stick",
  "group": "Stick",
  "label": "Weapon Select: fwd",
  "button": 12
 },
 {
  "id": "S.wpn_aft",
  "dev": "stick",
  "group": "Stick",
  "label": "Weapon Select: aft",
  "button": 14
 },
 {
  "id": "S.wpn_left",
  "dev": "stick",
  "group": "Stick",
  "label": "Weapon Select: left",
  "button": 15
 },
 {
  "id": "S.wpn_right",
  "dev": "stick",
  "group": "Stick",
  "label": "Weapon Select: right",
  "button": 13
 },
 {
  "id": "S.wpn_press",
  "dev": "stick",
  "group": "Stick",
  "label": "Weapon Select: press",
  "button": 11
 },
 {
  "id": "S.undesignate",
  "dev": "stick",
  "group": "Stick",
  "label": "Undesignate / NWS (small front button, index finger)",
  "button": 16
 },
 {
  "id": "S.paddle",
  "dev": "stick",
  "group": "Stick",
  "label": "Paddle, lower front",
  "button": 17
 },
 {
  "id": "S.recce",
  "dev": "stick",
  "group": "Stick",
  "label": "Rear hat: press (RECCE / Event Mark)",
  "button": 8
 },
 {
  "id": "S.trim_up",
  "dev": "stick",
  "group": "Stick",
  "label": "Trim hat: up (forward)",
  "hat": "up"
 },
 {
  "id": "S.trim_down",
  "dev": "stick",
  "group": "Stick",
  "label": "Trim hat: down (towards you)",
  "hat": "down"
 },
 {
  "id": "S.trim_left",
  "dev": "stick",
  "group": "Stick",
  "label": "Trim hat: left",
  "hat": "left"
 },
 {
  "id": "S.trim_right",
  "dev": "stick",
  "group": "Stick",
  "label": "Trim hat: right",
  "hat": "right"
 },
 {
  "id": "S.trim_press",
  "dev": "stick",
  "group": "Stick",
  "label": "Trim hat: press",
  "button": 1
 },
 {
  "id": "S.hat2_up",
  "dev": "stick",
  "group": "Stick",
  "label": "Rear hat (around 8): up",
  "button": 18
 },
 {
  "id": "S.hat2_down",
  "dev": "stick",
  "group": "Stick",
  "label": "Rear hat: down",
  "button": 20
 },
 {
  "id": "S.hat2_left",
  "dev": "stick",
  "group": "Stick",
  "label": "Rear hat: left",
  "button": 21
 },
 {
  "id": "S.hat2_right",
  "dev": "stick",
  "group": "Stick",
  "label": "Rear hat: right",
  "button": 19
 },
 {
  "id": "T.extlights_fwd",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Exterior Lights: fwd",
  "button": 3
 },
 {
  "id": "T.extlights_ctr",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Exterior Lights: centre",
  "button": 4
 },
 {
  "id": "T.extlights_aft",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Exterior Lights: aft",
  "button": 5
 },
 {
  "id": "T.raid",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "RAID/FLIR FOV button",
  "button": 6
 },
 {
  "id": "T.atc",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "ATC button",
  "button": 7
 },
 {
  "id": "T.disp_fwd",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Dispense (4-way): fwd",
  "button": 10
 },
 {
  "id": "T.disp_aft",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Dispense: aft",
  "button": 12
 },
 {
  "id": "T.disp_left",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Dispense: left",
  "button": 9
 },
 {
  "id": "T.disp_right",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Dispense: right",
  "button": 11
 },
 {
  "id": "T.disp_press",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Dispense: press",
  "button": 8
 },
 {
  "id": "T.comm_fwd",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "COMM (4-way): fwd",
  "button": 15
 },
 {
  "id": "T.comm_aft",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "COMM: aft",
  "button": 17
 },
 {
  "id": "T.comm_left",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "COMM: left",
  "button": 14
 },
 {
  "id": "T.comm_right",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "COMM: right",
  "button": 16
 },
 {
  "id": "T.comm_press",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "COMM: press",
  "button": 13
 },
 {
  "id": "T.spdbrk_fwd",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Speed Brake: fwd",
  "button": 18
 },
 {
  "id": "T.spdbrk_ctr",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Speed Brake: centre",
  "button": 19
 },
 {
  "id": "T.spdbrk_aft",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Speed Brake: aft",
  "button": 20
 },
 {
  "id": "T.cage",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "Cage/Uncage button",
  "button": 21
 },
 {
  "id": "T.tdc_press",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "TDC (mini-stick): press",
  "button": 22
 },
 {
  "id": "T.tdc_up",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "TDC pushed up (button)",
  "button": 26
 },
 {
  "id": "T.tdc_down",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "TDC pushed down (button)",
  "button": 28
 },
 {
  "id": "T.tdc_left",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "TDC pushed left (button)",
  "button": 29
 },
 {
  "id": "T.tdc_right",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "TDC pushed right (button)",
  "button": 27
 },
 {
  "id": "T.finger_up",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "3-position wheel under the left index finger: position 23",
  "button": 23
 },
 {
  "id": "T.finger_ctr",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "3-position wheel under the left index finger: centre",
  "button": 24
 },
 {
  "id": "T.finger_down",
  "dev": "throttle",
  "group": "Throttle handles",
  "label": "3-position wheel under the left index finger: position 25",
  "button": 25
 },
 {
  "id": "B.launchbar_1",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "LAUNCH BAR: up (RETRACT)",
  "button": 65
 },
 {
  "id": "B.launchbar_2",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "LAUNCH BAR: down (EXTEND)",
  "button": 66
 },
 {
  "id": "B.hook_up",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "HOOK: up",
  "button": 67
 },
 {
  "id": "B.hook_down",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "HOOK: down (DN)",
  "button": 68
 },
 {
  "id": "B.wingfold_1",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "WING FOLD (rotary knob): FOLD",
  "button": 69
 },
 {
  "id": "B.wingfold_2",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "WING FOLD: HOLD",
  "button": 70
 },
 {
  "id": "B.wingfold_3",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "WING FOLD: SPREAD",
  "button": 71
 },
 {
  "id": "B.wingfold_pull",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "WING FOLD: press (PUSH REL/LOCK)",
  "button": 72
 },
 {
  "id": "B.gear_up",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "LDG GEAR: up (UP)",
  "button": 73
 },
 {
  "id": "B.gear_down",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "LDG GEAR: down (DN)",
  "button": 74
 },
 {
  "id": "B.parkbrake_1",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "PARK BRK: up (PARK)",
  "button": 75
 },
 {
  "id": "B.parkbrake_2",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "PARK BRK: down (OFF)",
  "button": 76
 },
 {
  "id": "B.flaps_1",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "FLAP: up (AUTO)",
  "button": 77
 },
 {
  "id": "B.flaps_2",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "FLAP: centre (HALF)",
  "button": 78
 },
 {
  "id": "B.flaps_3",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "FLAP: down (FULL)",
  "button": 79
 },
 {
  "id": "B.red_button",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "Unlabelled round button (next to the Slider wheel)",
  "button": 80
 },
 {
  "id": "B.ag",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "A/G lighted button",
  "button": 81
 },
 {
  "id": "B.aa",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "A/A lighted button",
  "button": 82
 },
 {
  "id": "B.hmd_ccw",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "HMD knob: click − (left)",
  "button": 83,
  "pulse": true
 },
 {
  "id": "B.hmd_cw",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "HMD knob: click + (right)",
  "button": 84,
  "pulse": true
 },
 {
  "id": "B.hmd_press",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "HMD knob: PUSH",
  "button": 85
 },
 {
  "id": "B.aproll_1",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "ROLL switch: up (HDG SEL)",
  "button": 86
 },
 {
  "id": "B.aproll_2",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "ROLL switch: centre (ATT HOLD)",
  "button": 87
 },
 {
  "id": "B.aproll_3",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "ROLL switch: down (STRG SEL)",
  "button": 88
 },
 {
  "id": "B.appitch_1",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "PITCH switch: up (ALT HOLD)",
  "button": 89
 },
 {
  "id": "B.appitch_2",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "PITCH switch: centre (A/P OFF)",
  "button": 90
 },
 {
  "id": "B.appitch_3",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "PITCH switch: down (ATT HOLD)",
  "button": 91
 },
 {
  "id": "B.adv_mode",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "ADV MODE button (EMG/DIS)",
  "button": 92
 },
 {
  "id": "B.masterarm_1",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "MASTER ARM: up (ARM)",
  "button": 93
 },
 {
  "id": "B.masterarm_2",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "MASTER ARM: centre (SAFE)",
  "button": 94
 },
 {
  "id": "B.masterarm_3",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "MASTER ARM: down (SIM)",
  "button": 95
 },
 {
  "id": "B.jett",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "JETT button (under the striped cover)",
  "button": 96
 },
 {
  "id": "B.hdg_ccw",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "HDG knob: click −",
  "button": 97,
  "pulse": true
 },
 {
  "id": "B.hdg_cw",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "HDG knob: click +",
  "button": 98,
  "pulse": true
 },
 {
  "id": "B.hdg_press",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "HDG knob: PUSH",
  "button": 99
 },
 {
  "id": "B.crs_ccw",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "CRS knob: click −",
  "button": 100,
  "pulse": true
 },
 {
  "id": "B.crs_cw",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "CRS knob: click +",
  "button": 101,
  "pulse": true
 },
 {
  "id": "B.crs_press",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "CRS knob: PUSH",
  "button": 102
 },
 {
  "id": "B.panel_ccw",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "PANEL LTS knob: click −",
  "button": 103,
  "pulse": true
 },
 {
  "id": "B.panel_cw",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "PANEL LTS knob: click +",
  "button": 104,
  "pulse": true
 },
 {
  "id": "B.panel_press",
  "dev": "throttle",
  "group": "Throttle base",
  "label": "PANEL LTS knob: PUSH",
  "button": 105
 }
];
export const AXIS_CONTROLS: AxisControl[] = [
 {
  "dev": "stick",
  "axis": "x",
  "group": "Stick",
  "label": "Stick left/right"
 },
 {
  "dev": "stick",
  "axis": "y",
  "group": "Stick",
  "label": "Stick fwd/aft"
 },
 {
  "dev": "pedals",
  "axis": "z",
  "group": "Pedals",
  "label": "Pedals (yaw)"
 },
 {
  "dev": "pedals",
  "axis": "y",
  "group": "Pedals",
  "label": "Left brake (toe)"
 },
 {
  "dev": "pedals",
  "axis": "x",
  "group": "Pedals",
  "label": "Right brake (toe)"
 },
 {
  "dev": "throttle",
  "axis": "ry",
  "group": "Throttle handles",
  "label": "Left throttle (RY)"
 },
 {
  "dev": "throttle",
  "axis": "rx",
  "group": "Throttle handles",
  "label": "Right throttle (RX)"
 },
 {
  "dev": "throttle",
  "axis": "x",
  "group": "Throttle handles",
  "label": "TDC left/right"
 },
 {
  "dev": "throttle",
  "axis": "y",
  "group": "Throttle handles",
  "label": "TDC up/down"
 },
 {
  "dev": "throttle",
  "axis": "z",
  "group": "Throttle handles",
  "label": "Radar elevation wheel (handle)"
 },
 {
  "dev": "throttle",
  "axis": "slider",
  "group": "Throttle base",
  "label": "Base \"Slider\" wheel (top right)"
 },
 {
  "dev": "throttle",
  "axis": "dial",
  "group": "Throttle base",
  "label": "Base \"Dial\" wheel (bottom right)"
 }
];
