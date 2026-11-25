# Wingman

Play Switch games that run natively on Apple silicon Macs through [nxnative](https://github.com/seedds/nxnative), using a flight-sim HOTAS (stick and throttle) instead of a gamepad.

Wingman reads your stick and throttle, turns them into one Switch controller and sends it to the running game over localhost. The first profile is **Star Fox** (Switch 2), with defaults for the **WinWing** F/A-18 grip and Orion 2 throttle. Any HID joystick or throttle works: press **Learn** and move the control you want.

```
HOTAS (IOKit HID) ──► Wingman engine (500 Hz) ──UDP 127.0.0.1:47821──► nxnative host (DM_EXT_PAD) ──► game, player 1
```

> **Status: early (0.1.0).** The mapping engine has unit tests. Full in-game testing with the HOTAS is still in progress, so expect to adjust a few bindings.

## What it does

- **Rapid fire:** holding the trigger fires repeatedly (12 presses a second by default; you can change the rate and the press length).
- **Charged shot:** the second trigger detent holds the fire button to charge, and easing back fires the charged shot. When Hold and Rapid fire share a button, Hold wins, and rapid fire restarts with a release.
- **One-press sequences:** a single press can send a timed sequence, for example a double-tap L for a barrel roll.
- **Throttle zones:** each part of the throttle's travel can hold a button. By default, back is brake and forward is boost, with hysteresis so the lever doesn't flicker at the edge of a zone.
- **Stick shaping:** each axis has invert, deadzone, curve and scale settings, with live meters.
- **Learn:** press or move a control on the HOTAS to bind it.
- **Live view:** shows what the game receives (buttons and both sticks).
- **Game setup:** **▶ Play** turns on controller input in the game's `env.txt` and launches the game.

Changes save on their own.

## Requirements

- An Apple silicon Mac with macOS 12 or later, and a game nxnative can build. nxnative's own requirements apply (Xcode, the Metal toolchain and so on; see its README).
- A copy of the game that you dumped yourself from your own console. This repo contains no game files, keys or firmware, and never will.
- A HOTAS or any HID joystick. The defaults are for WinWing hardware, and everything else can be bound with Learn.

## Setup

### 1. Add the receiver to nxnative

Stock nxnative has no way to accept a controller from another app, so add a small receiver to its Mac host: one new file and two lines. The steps are in [`nxnative-patch/`](nxnative-patch/README.md):

```bash
cd nxnative-patch
NXNATIVE=~/nxnative                  # your nxnative checkout
cp ext_pad_mac.h "$NXNATIVE/dm/host/"
git -C "$NXNATIVE" apply "$PWD/front_mac.m.patch"
```

The receiver stays off unless the game's `env.txt` has `DM_EXT_PAD=<port>` in it, so a game built with the patch runs exactly as before until you turn it on.

### 2. Build the game with nxnative

Build it the usual way, as a Mac app:

```bash
python3 scripts/build_ipa_from_nsp.py "<folder with your game>" --platform mac-app
```

If you built the game before you applied the patch, build it again. The extracted files are reused, so only the host and the packaging run again.

### 3. Install Wingman

**Download:** get `Wingman-<version>-macos-arm64.zip` from [Releases](../../releases), unzip it, and move `Wingman.app` to Applications.

The app is ad-hoc signed, not notarized, so macOS blocks it the first time. Either right-click it, choose **Open** and confirm, or run:

```bash
xattr -dr com.apple.quarantine /Applications/Wingman.app
```

**Or build it yourself** (needs Node 20+ and [Rust](https://rustup.rs)):

```bash
npm ci
./build-app.sh          # type check, tests, release build, installs to /Applications
```

### 4. Play

1. Open Wingman. The **Stick** and **Throttle** chips turn green when it sees your devices. If macOS asks for **Input Monitoring** access, allow it.
2. Wingman looks for the game in `/Applications` and `~/Applications`. If yours is elsewhere, for example on an external drive, click **Choose…** under **Game**.
3. Quit the game if it's running, then click **▶ Play**. Once the game is up, the chip reads **In game · player 1**.

Play writes two lines to the game's `~/Library/Application Support/nxnative/<title id>/env.txt` and keeps every other line as it was:

- `DM_EXT_PAD=47821` turns on the receiver.
- `DM_HID_PADS=0` stops the game from also reading the HOTAS as raw gamepads, which would add extra players.

**Turn off** in the Game card removes both lines again.

## Star Fox defaults (WinWing)

Game buttons follow the game's published control list. Every row can be changed in the app.

| HOTAS | Star Fox | Mode |
| --- | --- | --- |
| Stick X / Y | Steer, climb / dive (left stick, Y inverted) | axis, light curve |
| Trigger, 1st detent | A: fire | rapid fire |
| Trigger, 2nd detent | A: charge, fires on release | hold |
| Weapon release (pickle) | Y: bomb | tap |
| Throttle lever back (0–15 %) | ZL: brake | zone |
| Throttle lever forward (85–100 %) | ZR: boost | zone |
| COMM switch left / right | Double-tap L / R: barrel roll | one-press sequence |
| TDC mini-stick | Right stick | axis |
| TDC press | R3 | hold |
| Trim hat | D-pad | hold |
| Trim hat press | A (menus) | hold |
| Paddle | B (back) | hold |
| Cage | + (pause) | tap |

If the throttle zones come out the wrong way round (full forward brakes), tick **Invert** on the zones.

## Other hardware

Devices get a role from their product name: names containing *THROTTLE* become `throttle`, *RUDDER* or *PEDAL* become `pedals`, and *JOYSTICK*, *STICK* or *GRIP* become `stick`. Anything else keeps its product name. Bindings store the role plus the HID button number, hat direction or axis, so:

- for a non-WinWing stick, press **Learn** on each row and use the control you want;
- the WinWing names in the drop-downs are labels only, and buttons from other devices appear as "<device> button <n>".

## How it works

| Piece | File |
| --- | --- |
| HID reader: joysticks and gamepads, not seized, so other apps can read them too | `src-tauri/src/hid.rs` |
| Mapping engine: a pure `Engine::tick`, unit-tested, plus the 500 Hz sender | `src-tauri/src/engine.rs` |
| Profile model and the Star Fox defaults | `src-tauri/src/profile.rs` |
| The game's `env.txt`, finding and launching the app | `src-tauri/src/game.rs` |
| UI (Svelte 5) | `src/App.svelte`, `src/lib/` |
| nxnative receiver | `nxnative-patch/ext_pad_mac.h` |

Profiles are stored in `~/Library/Application Support/Wingman/profiles/<title id>.json`.

### Wire format (NXEP v1, little endian, 36 bytes)

| Bytes | Field |
| --- | --- |
| 0–3 | `"NXEP"` |
| 4–5 | u16 version = 1 |
| 6–7 | u16 flags (bit 0 = leaving) |
| 8–11 | u32 sequence |
| 12–19 | u64 buttons, NpadButton bits: A B X Y LS RS L R ZL ZR + − ← ↑ → ↓ |
| 20–35 | i32 LX LY RX RY, ±32767, y up |

The receiver joins as the next free player the first time a packet arrives. About once a second it answers `"NXEA"`, u16 version, u16 player. If no packet arrives for 300 ms, it releases every button. Any program can drive a game this way, not only Wingman.

## Development

```bash
npm ci
npx tauri dev                    # run with hot reload
cd src-tauri && cargo test       # engine, profile, env.txt and HID-role tests
npx svelte-check                 # UI type check
./build-app.sh --no-install      # release build only
```

## Troubleshooting

- **"Game not running":** start it with **▶ Play**, or check the app path in the Game card.
- **"Game not listening":** the game started before controller input was turned on, or it was built without the patch. Quit it and press Play again. The game's log (`~/Library/Logs/nxnative/<title id>.log`) should contain `external controller: listening on 127.0.0.1:47821`.
- **Chips stay grey:** check that macOS gave Wingman Input Monitoring access, and that the device shows up in System Information under USB.
- **Shots get dropped with rapid fire:** lower the rate. Each press and each release has to last at least one game frame.

## Disclaimer

Wingman isn't affiliated with Nintendo, WinWing or the nxnative project. All trademarks belong to their owners. Use it only with games you own.

## License

MIT. See [LICENSE](LICENSE).
