<script lang="ts">
  // What the game sees: the Switch controller Wingman is sending, live.
  let { pressed = [], axes = [0, 0, 0, 0] }: { pressed?: string[]; axes?: number[] } = $props();
  const on = (b: string) => pressed.includes(b);
  const dot = (x: number, y: number) => `left:${50 + (x / 32767) * 38}%;top:${50 - (y / 32767) * 38}%`;
</script>

<div class="pad">
  <div class="shoulders">
    <div class="sh"><span class:on={on('ZL')}>ZL</span><span class:on={on('L')}>L</span></div>
    <div class="sh"><span class:on={on('R')}>R</span><span class:on={on('ZR')}>ZR</span></div>
  </div>
  <div class="body">
    <div class="side">
      <div class="small"><span class:on={on('Minus')}>−</span></div>
      <div class="stick" class:on={on('LS')} title="Left stick"><i style={dot(axes[0], axes[1])}></i></div>
      <div class="dpad">
        <span class="u" class:on={on('Up')}>▲</span><span class="l" class:on={on('Left')}>◀</span>
        <span class="r" class:on={on('Right')}>▶</span><span class="d" class:on={on('Down')}>▼</span>
      </div>
    </div>
    <div class="side">
      <div class="small"><span class:on={on('Plus')}>+</span></div>
      <div class="abxy">
        <span class="x" class:on={on('X')}>X</span><span class="y" class:on={on('Y')}>Y</span>
        <span class="a" class:on={on('A')}>A</span><span class="b" class:on={on('B')}>B</span>
      </div>
      <div class="stick" class:on={on('RS')} title="Right stick"><i style={dot(axes[2], axes[3])}></i></div>
    </div>
  </div>
</div>

<style>
  .pad { background: linear-gradient(180deg, #171c28, #10131b); border: 1px solid var(--line-2); border-radius: 22px; padding: 12px 16px 16px; width: 100%; max-width: 380px; }
  .shoulders { display: flex; justify-content: space-between; margin-bottom: 10px; }
  .sh { display: flex; gap: 6px; }
  span { display: grid; place-items: center; min-width: 30px; height: 26px; padding: 0 6px; border-radius: 8px; font-size: 12px; font-weight: 700;
    background: rgba(255,255,255,.05); border: 1px solid var(--line); color: var(--muted); transition: background .05s, color .05s; }
  span.on { background: var(--accent); color: #111; border-color: var(--accent); box-shadow: 0 0 14px rgba(255,179,71,.55); }
  .body { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
  .side { display: grid; justify-items: center; gap: 10px; }
  .small span { min-width: 24px; height: 22px; border-radius: 50%; }
  .stick { position: relative; width: 74px; height: 74px; border-radius: 50%; background: radial-gradient(circle, #222a3a, #121620); border: 1px solid var(--line-2); }
  .stick.on { border-color: var(--accent); }
  .stick i { position: absolute; width: 18px; height: 18px; margin: -9px 0 0 -9px; border-radius: 50%; background: var(--ok); box-shadow: 0 0 12px var(--ok); }
  .dpad, .abxy { position: relative; width: 84px; height: 84px; }
  .dpad span, .abxy span { position: absolute; width: 28px; min-width: 0; height: 28px; padding: 0; }
  .abxy span { border-radius: 50%; }
  .u, .x { left: 28px; top: 0; } .d, .b { left: 28px; bottom: 0; } .l, .y { left: 0; top: 28px; } .r, .a { right: 0; top: 28px; }
</style>
