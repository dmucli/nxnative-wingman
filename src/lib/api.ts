import { invoke } from '@tauri-apps/api/core';
import { CONTROLS, AXIS_CONTROLS } from './controls';

// Mirrors src-tauri/src/profile.rs (serde camelCase).
export type ButtonSrc = { dev: string; button?: number; hat?: string };
export type AxisSrc = { dev: string; axis: string };
export type Step = { buttons: string[]; ms: number };
export type Mode = 'hold' | 'turbo' | 'tap' | 'macro';
export type Bind = { src: ButtonSrc; mode: Mode; buttons: string[]; steps: Step[]; label: string };
export type AxisMap = { src: AxisSrc; to: string; invert: boolean; deadzone: number; expo: number; scale: number; label: string };
export type Zone = { src: AxisSrc; invert: boolean; from: number; to: number; button: string; label: string };
export type Profile = {
  name: string; titleId: string; appPath: string; port: number;
  turboHz: number; turboDuty: number; tapMs: number;
  axes: AxisMap[]; zones: Zone[]; binds: Bind[];
};

export type DevState = { buttons: number[]; axes: Record<string, number>; hat: number };
export type Touched = { dev: string; button?: number; hat?: string; axis?: string; seq: number };
export type Live = {
  devices: string[];
  inputs: Record<string, DevState>;
  out: { buttons: number; axes: [number, number, number, number] };
  pressed: string[];
  link: { player: number; ackAgeMs: number; sending: boolean; error?: string | null };
  touched: Touched | null;
  hidError: number | null;
};
export type GameStatus = { envPath: string; enabled: boolean; appExists: boolean; running: boolean };

export const BUTTONS = ['A', 'B', 'X', 'Y', 'LS', 'RS', 'L', 'R', 'ZL', 'ZR', 'Plus', 'Minus', 'Left', 'Up', 'Right', 'Down'];
export const BUTTON_LABEL: Record<string, string> = { Plus: '+', Minus: '−', Left: '←', Up: '↑', Right: '→', Down: '↓', LS: 'L3', RS: 'R3' };
export const label = (b: string) => BUTTON_LABEL[b] ?? b;
export const STICK_AXES = ['LX', 'LY', 'RX', 'RY'];

export const api = {
  live: () => invoke<Live>('live'),
  getProfile: () => invoke<Profile>('get_profile'),
  defaultProfile: () => invoke<Profile>('default_profile'),
  saveProfile: (p: Profile) => invoke<void>('save_profile', { p }),
  gameStatus: () => invoke<GameStatus>('game_status'),
  setGameEnabled: (on: boolean) => invoke<void>('set_game_enabled', { on }),
  launchGame: () => invoke<void>('launch_game'),
  setSending: (on: boolean) => invoke<void>('set_sending', { on }),
  pickGameApp: () => invoke<Profile>('pick_game_app'),
};

export function controlName(s: ButtonSrc): string {
  const c = CONTROLS.find(c => c.dev === s.dev && (s.button != null ? c.button === s.button : c.hat === s.hat));
  if (c) return c.label;
  return s.button != null ? `${s.dev} button ${s.button}` : `${s.dev} hat ${s.hat}`;
}
export function controlGroup(s: ButtonSrc): string {
  return CONTROLS.find(c => c.dev === s.dev && (s.button != null ? c.button === s.button : c.hat === s.hat))?.group ?? s.dev;
}
export function axisName(s: AxisSrc): string {
  return AXIS_CONTROLS.find(a => a.dev === s.dev && a.axis === s.axis)?.label ?? `${s.dev} ${s.axis}`;
}
export function srcKey(s: ButtonSrc): string { return s.button != null ? `${s.dev}:b${s.button}` : `${s.dev}:h${s.hat}`; }

const HAT: Record<string, number[]> = { up: [7, 0, 1], right: [1, 2, 3], down: [3, 4, 5], left: [5, 6, 7] };
export function srcActive(live: Live | null, s: ButtonSrc): boolean {
  const d = live?.inputs[s.dev]; if (!d) return false;
  if (s.button != null) return d.buttons.includes(s.button);
  return !!s.hat && (HAT[s.hat] ?? []).includes(d.hat);
}
export function axisValue(live: Live | null, s: AxisSrc): number | undefined { return live?.inputs[s.dev]?.axes[s.axis]; }

/** Same shaping as engine.rs shape(): 0..1 → -1..1. */
export function shape(v: number, m: AxisMap): number {
  let c = v * 2 - 1; if (m.invert) c = -c;
  const dz = Math.min(Math.max(m.deadzone, 0), 0.9), a = Math.abs(c);
  c = a <= dz ? 0 : Math.sign(c) * (a - dz) / (1 - dz);
  const e = Math.min(Math.max(m.expo, 0), 1);
  c = (1 - e) * c + e * c * c * c;
  return Math.max(-1, Math.min(1, c * m.scale));
}
export const doubleTap = (b: string): Step[] => [{ buttons: [b], ms: 50 }, { buttons: [], ms: 50 }, { buttons: [b], ms: 50 }];
