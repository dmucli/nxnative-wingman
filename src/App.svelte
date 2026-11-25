<script lang="ts">
  import { onMount } from 'svelte';
  import SwitchPad from './lib/SwitchPad.svelte';
  import { CONTROLS, AXIS_CONTROLS } from './lib/controls';
  import {
    api, BUTTONS, STICK_AXES, label, controlName, controlGroup, axisName, srcKey, srcActive, axisValue, shape, doubleTap,
    type Profile, type Live, type GameStatus, type Bind, type Mode, type ButtonSrc, type AxisSrc,
  } from './lib/api';

  let profile = $state<Profile | null>(null);
  let live = $state<Live | null>(null);
  let game = $state<GameStatus | null>(null);
  let learn = $state<{ kind: 'bind' | 'axis' | 'zone'; i: number; seq: number } | null>(null);
  let note = $state('');
  let noteBad = $state(false);
  let saved = $state(true);

  const MODES: { v: Mode; t: string; d: string }[] = [
    { v: 'hold', t: 'Hold', d: 'Pressed while you hold the control' },
    { v: 'turbo', t: 'Rapid fire', d: 'Pressed and released repeatedly while held. A Hold of the same button wins over it' },
    { v: 'tap', t: 'Tap', d: 'One short press when the control goes down' },
    { v: 'macro', t: 'Sequence', d: 'A timed sequence from one press (e.g. a double-tap)' },
  ];
  const groups = [...new Set(CONTROLS.map(c => c.group))];
  const axisGroups = [...new Set(AXIS_CONTROLS.map(a => a.group))];

  function say(t: string, bad = false) { note = t; noteBad = bad; setTimeout(() => { if (note === t) note = ''; }, 6000); }

  onMount(() => {
    api.getProfile().then(p => (profile = p));
    const poll = setInterval(async () => {
      try { live = await api.live(); } catch { return; }
      const t = live.touched;
      if (learn && profile && t && t.seq > learn.seq) {
        if (learn.kind === 'bind' && (t.button != null || t.hat)) {
          profile.binds[learn.i].src = t.button != null ? { dev: t.dev, button: t.button } : { dev: t.dev, hat: t.hat };
          learn = null;
        } else if (learn.kind !== 'bind' && t.axis) {
          const src = { dev: t.dev, axis: t.axis };
          if (learn.kind === 'axis') profile.axes[learn.i].src = src; else profile.zones[learn.i].src = src;
          learn = null;
        }
      }
    }, 33);
    const slow = setInterval(async () => { try { game = await api.gameStatus(); } catch { /* not in Tauri */ } }, 1000);
    api.gameStatus().then(g => (game = g)).catch(() => {});
    return () => { clearInterval(poll); clearInterval(slow); };
  });

  // Save every change (debounced).
  let timer: ReturnType<typeof setTimeout> | undefined;
  let first = true;
  $effect(() => {
    if (!profile) return;
    const snap = JSON.stringify(profile);
    if (first) { first = false; return; }
    saved = false;
    clearTimeout(timer);
    timer = setTimeout(() => {
      api.saveProfile(JSON.parse(snap)).then(() => (saved = true)).catch(e => say(`Not saved: ${e}`, true));
    }, 300);
  });

  function startLearn(kind: 'bind' | 'axis' | 'zone', i: number) { learn = { kind, i, seq: live?.touched?.seq ?? 0 }; }
  function setSrcFromKey(b: Bind, key: string) {
    const c = CONTROLS.find(c => srcKey(c as ButtonSrc) === key);
    if (c) b.src = c.button != null ? { dev: c.dev, button: c.button } : { dev: c.dev, hat: c.hat };
  }
  function setAxisFromKey(o: { src: AxisSrc }, key: string) { const [dev, axis] = key.split(':'); o.src = { dev, axis }; }
  function toggle(list: string[], b: string) { const i = list.indexOf(b); if (i >= 0) list.splice(i, 1); else list.push(b); }
  function setMode(b: Bind, m: Mode) {
    b.mode = m;
    if (m === 'macro' && b.steps.length === 0) b.steps = doubleTap(b.buttons[0] ?? 'A');
    if (m !== 'macro' && b.buttons.length === 0) b.buttons = ['A'];
  }
  function addBind() { profile!.binds.push({ src: { dev: 'stick', button: 16 }, mode: 'hold', buttons: ['A'], steps: [], label: '' }); startLearn('bind', profile!.binds.length - 1); }
  async function resetAll() {
    if (!confirm('Put every Star Fox binding back to the defaults?')) return;
    profile = await api.defaultProfile(); say('Back to the defaults');
  }
  async function play() {
    try { await api.launchGame(); say(game?.running ? 'The game is already running: quit and start it again if it does not show "player" above' : 'Starting the game…'); }
    catch (e) { say(String(e), true); }
  }
  async function chooseApp() {
    try { const p = await api.pickGameApp(); if (p.appPath !== profile?.appPath) { profile = p; say('Game app set'); } }
    catch (e) { say(String(e), true); }
  }
  async function toggleSending() { await api.setSending(!live?.link.sending); }

  const hasDev = (d: string) => live?.devices.includes(d) ?? false;
  const pct = (v: number | undefined) => `${Math.round((v ?? 0) * 100)}%`;
</script>

<header>
  <div class="brand"><div class="logo"></div>Wingman <small>{profile?.name ?? ''}</small></div>
  <div class="chips">
    <span class="chip" class:on={hasDev('stick')}><i></i>Stick</span>
    <span class="chip" class:on={hasDev('throttle')}><i></i>Throttle</span>
    {#if live?.link.player}
      <span class="chip on"><i></i>In game · player {live.link.player}</span>
    {:else if game?.running}
      <span class="chip mid" title="The game runs but does not answer: it was started before Wingman turned its controller input on, or it is an older build"><i></i>Game not listening</span>
    {:else}
      <span class="chip"><i></i>Game not running</span>
    {/if}
  </div>
  <div class="spacer"></div>
  {#if note}<span class="note" class:bad={noteBad}>{note}</span>{/if}
  <span class="saved">{saved ? 'Saved' : 'Saving…'}</span>
  <button class="btn" class:off={!live?.link.sending} onclick={toggleSending} title="Stop sending to release every button in the game">
    {live?.link.sending ? 'Sending' : 'Paused'}
  </button>
  <button class="btn primary" onclick={play} disabled={!game?.appExists}>▶ Play {profile?.name ?? ''}</button>
</header>

{#if live?.hidError}<div class="banner">The HOTAS cannot be read (IOHIDManagerOpen {live.hidError}). Allow Wingman under System Settings → Privacy & Security → Input Monitoring, then restart it.</div>{/if}

{#if profile}
<main>
  <section class="col">
    <div class="card">
      <div class="head"><h2>Buttons</h2><span class="hint">Click <b>Learn</b> and press a control on the HOTAS. Rapid fire and Hold on the same button: Hold wins, and letting it go fires the charged shot.</span>
        <button class="btn sm" onclick={addBind}>+ Add</button></div>
      <div class="rows">
        {#each profile.binds as b, i (i)}
          <div class="row" class:live={srcActive(live, b.src)}>
            <i class="dot"></i>
            <div class="src">
              <select value={srcKey(b.src)} onchange={e => setSrcFromKey(b, (e.target as HTMLSelectElement).value)} title={controlName(b.src)}>
                {#if !CONTROLS.some(c => srcKey(c as ButtonSrc) === srcKey(b.src))}<option value={srcKey(b.src)}>{controlName(b.src)}</option>{/if}
                {#each groups as g}
                  <optgroup label={g}>{#each CONTROLS.filter(c => c.group === g) as c}<option value={srcKey(c as ButtonSrc)}>{c.label}</option>{/each}</optgroup>
                {/each}
              </select>
              <small>{controlGroup(b.src)}</small>
            </div>
            <button class="btn sm" class:learning={learn?.kind === 'bind' && learn.i === i} onclick={() => startLearn('bind', i)}>{learn?.kind === 'bind' && learn.i === i ? 'Press…' : 'Learn'}</button>
            <select class="mode" value={b.mode} onchange={e => setMode(b, (e.target as HTMLSelectElement).value as Mode)} title={MODES.find(m => m.v === b.mode)?.d}>
              {#each MODES as m}<option value={m.v}>{m.t}</option>{/each}
            </select>
            <div class="target">
              {#if b.mode !== 'macro'}
                <div class="bset">{#each BUTTONS as s}<button class="k" class:sel={b.buttons.includes(s)} onclick={() => toggle(b.buttons, s)}>{label(s)}</button>{/each}</div>
              {:else}
                <div class="steps">
                  {#each b.steps as st, j}
                    <div class="step">
                      <span class="n">{j + 1}</span>
                      <div class="bset">{#each BUTTONS as s}<button class="k" class:sel={st.buttons.includes(s)} onclick={() => toggle(st.buttons, s)}>{label(s)}</button>{/each}</div>
                      <input type="number" min="10" max="2000" step="10" bind:value={st.ms} /><span class="u">ms</span>
                      <button class="x" onclick={() => b.steps.splice(j, 1)} title="Remove step">×</button>
                    </div>
                  {/each}
                  <div class="presets">
                    <button class="btn sm" onclick={() => b.steps.push({ buttons: [], ms: 50 })}>+ Step</button>
                    <button class="btn sm" onclick={() => (b.steps = doubleTap('L'))}>Double-tap L</button>
                    <button class="btn sm" onclick={() => (b.steps = doubleTap('R'))}>Double-tap R</button>
                  </div>
                </div>
              {/if}
            </div>
            <input class="lbl" placeholder="What it does" bind:value={b.label} />
            <button class="x" onclick={() => profile!.binds.splice(i, 1)} title="Remove">×</button>
          </div>
        {/each}
      </div>
    </div>

    <div class="card">
      <div class="head"><h2>Sticks</h2><span class="hint">Stick forward should push the Switch stick up (Y inverted). Flip <b>Invert</b> if the game feels upside down.</span>
        <button class="btn sm" onclick={() => profile!.axes.push({ src: { dev: 'stick', axis: 'x' }, to: 'LX', invert: false, deadzone: 0.03, expo: 0, scale: 1, label: '' })}>+ Add</button></div>
      <div class="rows">
        {#each profile.axes as a, i (i)}
          {@const v = axisValue(live, a.src)}
          <div class="row axis">
            <div class="src">
              <select value={`${a.src.dev}:${a.src.axis}`} onchange={e => setAxisFromKey(a, (e.target as HTMLSelectElement).value)}>
                {#if !AXIS_CONTROLS.some(c => c.dev === a.src.dev && c.axis === a.src.axis)}<option value={`${a.src.dev}:${a.src.axis}`}>{axisName(a.src)}</option>{/if}
                {#each axisGroups as g}<optgroup label={g}>{#each AXIS_CONTROLS.filter(c => c.group === g) as c}<option value={`${c.dev}:${c.axis}`}>{c.label}</option>{/each}</optgroup>{/each}
              </select>
              <small>{a.label}</small>
            </div>
            <button class="btn sm" class:learning={learn?.kind === 'axis' && learn.i === i} onclick={() => startLearn('axis', i)}>{learn?.kind === 'axis' && learn.i === i ? 'Move…' : 'Learn'}</button>
            <span class="arrow">→</span>
            <select class="mode" bind:value={a.to}>{#each STICK_AXES as s}<option value={s}>{s === 'LX' ? 'Left stick X' : s === 'LY' ? 'Left stick Y' : s === 'RX' ? 'Right stick X' : 'Right stick Y'}</option>{/each}</select>
            <label class="tog"><input type="checkbox" bind:checked={a.invert} /> Invert</label>
            <label class="sl">Dead zone <input type="range" min="0" max="0.3" step="0.01" bind:value={a.deadzone} /><b>{pct(a.deadzone)}</b></label>
            <label class="sl">Curve <input type="range" min="0" max="1" step="0.05" bind:value={a.expo} /><b>{pct(a.expo)}</b></label>
            <label class="sl">Scale <input type="range" min="0.2" max="2" step="0.05" bind:value={a.scale} /><b>{pct(a.scale)}</b></label>
            <div class="meters">
              <div class="meter" title="HOTAS"><i style={`left:${(v ?? 0.5) * 100}%`}></i></div>
              <div class="meter out" title="Sent to the game"><i style={`left:${(shape(v ?? 0.5, a) + 1) * 50}%`}></i></div>
            </div>
            <button class="x" onclick={() => profile!.axes.splice(i, 1)} title="Remove">×</button>
          </div>
        {/each}
      </div>
    </div>

    <div class="card">
      <div class="head"><h2>Throttle zones</h2><span class="hint">A button held while the lever is inside a zone. Move the throttle and watch the marker; flip <b>Invert</b> if forward reads as 0%.</span>
        <button class="btn sm" onclick={() => profile!.zones.push({ src: { dev: 'throttle', axis: 'ry' }, invert: false, from: 0.4, to: 0.6, button: 'A', label: '' })}>+ Add</button></div>
      <div class="rows">
        {#each profile.zones as z, i (i)}
          {@const raw = axisValue(live, z.src)}
          {@const v = raw == null ? null : z.invert ? 1 - raw : raw}
          {@const inside = v != null && v >= z.from && v <= z.to}
          <div class="row zone" class:live={inside}>
            <i class="dot"></i>
            <div class="src">
              <select value={`${z.src.dev}:${z.src.axis}`} onchange={e => setAxisFromKey(z, (e.target as HTMLSelectElement).value)}>
                {#each axisGroups as g}<optgroup label={g}>{#each AXIS_CONTROLS.filter(c => c.group === g) as c}<option value={`${c.dev}:${c.axis}`}>{c.label}</option>{/each}</optgroup>{/each}
              </select>
              <small>{z.label}</small>
            </div>
            <button class="btn sm" class:learning={learn?.kind === 'zone' && learn.i === i} onclick={() => startLearn('zone', i)}>{learn?.kind === 'zone' && learn.i === i ? 'Move…' : 'Learn'}</button>
            <label class="tog"><input type="checkbox" bind:checked={z.invert} /> Invert</label>
            <div class="zbar">
              <div class="band" style={`left:${z.from * 100}%;width:${Math.max(0, z.to - z.from) * 100}%`}></div>
              {#if v != null}<i style={`left:${v * 100}%`}></i>{/if}
            </div>
            <label class="sl">From <input type="range" min="0" max="1" step="0.01" bind:value={z.from} /><b>{pct(z.from)}</b></label>
            <label class="sl">To <input type="range" min="0" max="1" step="0.01" bind:value={z.to} /><b>{pct(z.to)}</b></label>
            <select class="mode" bind:value={z.button}>{#each BUTTONS as s}<option value={s}>{label(s)}</option>{/each}</select>
            <input class="lbl" placeholder="What it does" bind:value={z.label} />
            <button class="x" onclick={() => profile!.zones.splice(i, 1)} title="Remove">×</button>
          </div>
        {/each}
      </div>
    </div>
  </section>

  <aside class="col side">
    <div class="card">
      <div class="head"><h2>What the game sees</h2></div>
      <SwitchPad pressed={live?.pressed ?? []} axes={live?.out.axes ?? [0, 0, 0, 0]} />
    </div>
    <div class="card">
      <div class="head"><h2>Rapid fire</h2></div>
      <label class="sl wide">Rate <input type="range" min="4" max="25" step="1" bind:value={profile.turboHz} /><b>{profile.turboHz}/s</b></label>
      <label class="sl wide">Pressed <input type="range" min="0.2" max="0.8" step="0.05" bind:value={profile.turboDuty} /><b>{pct(profile.turboDuty)}</b></label>
      <label class="sl wide">Tap length <input type="range" min="20" max="200" step="10" bind:value={profile.tapMs} /><b>{profile.tapMs} ms</b></label>
      <p class="hint">If shots get lost, lower the rate (each press and each release must last at least one game frame).</p>
    </div>
    <div class="card">
      <div class="head"><h2>Game</h2></div>
      <div class="field">App <input bind:value={profile.appPath} placeholder="Not found yet: choose it" title={profile.appPath} /><button class="btn sm" onclick={chooseApp}>Choose…</button></div>
      <label class="field">Port <input type="number" min="1024" max="65535" bind:value={profile.port} /></label>
      {#if game}
        <p class="hint">Controller input in <code>{game.envPath}</code>: <b>{game.enabled ? 'on' : 'off'}</b>.
          {#if game.enabled}<button class="link" onclick={() => api.setGameEnabled(false)}>Turn off</button>{:else}<button class="link" onclick={() => api.setGameEnabled(true)}>Turn on</button>{/if}
          Play turns it on. It also stops the game reading the HOTAS itself (<code>DM_HID_PADS=0</code>), so start the game from here or after turning it on.</p>
        {#if !game.appExists}<p class="hint bad">{profile.appPath ? 'The app is not there. Is its drive connected?' : 'Choose the Star Fox app that nxnative built.'}</p>{/if}
      {/if}
      <button class="btn sm" onclick={resetAll}>Reset to defaults</button>
    </div>
  </aside>
</main>
{/if}
