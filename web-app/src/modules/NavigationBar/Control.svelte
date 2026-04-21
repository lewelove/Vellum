<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";

  let isPlaying = $derived(player.state === "play");

  async function togglePlay() { 
    try { await fetch('/api/toggle-pause', { method: 'POST' }); } catch(e) {} 
  }
  
  async function next() { 
    try { await fetch('/api/next', { method: 'POST' }); } catch(e) {} 
  }
  
  async function prev() { 
    try { await fetch('/api/prev', { method: 'POST' }); } catch(e) {} 
  }

  let tickingElapsed = $state(0);
  let duration = $derived(player.duration || 0);
  let progress = $derived(duration > 0 ? (tickingElapsed / duration) * 100 : 0);

  let phase = $state(0);
  let height = $state(0);
  let lastTime = performance.now();

  function tick() {
    const now = performance.now();
    const dt = (now - lastTime) / 1000;
    lastTime = now;

    if (player.state === "play") {
      const delta = (now - player.lastUpdated) / 1000;
      tickingElapsed = Math.min(player.elapsed + delta, player.duration || 0);
      phase -= dt * -0.0; 
    } else {
      tickingElapsed = player.elapsed || 0;
    }
    
    requestAnimationFrame(tick);
  }

  onMount(() => {
    lastTime = performance.now();
    const raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });

  let pathD = $derived.by(() => {
    if (height <= 0) return "";
    
    const midX = 12;
    const pad = 2; 
    const straightLen = 0;
    const waveLength = 20;
    const maxAmplitude = 2;
    const transitionLen = 100;
    const thickness = 4;
    const R = thickness / 2;
    
    const waveStart = straightLen + pad;
    const waveEnd = height - straightLen - pad;
    const waveHeight = Math.max(0, waveEnd - waveStart);
    const actualTransition = Math.min(transitionLen, waveHeight / 2);
    
    const pts =[];
    const step = 1;

    if (height <= straightLen * 2 + pad * 2) {
      for (let y = pad; y <= height - pad; y += step) {
        pts.push({ x: midX, y });
      }
    } else {
      for (let y = pad; y <= height - pad; y += step) {
        let x = midX;
        if (y > waveStart && y < waveEnd) {
          const prog = (y - waveStart) / waveHeight;
          const distStart = y - waveStart;
          const distEnd = waveEnd - y;
          
          let envelope = 1;
          if (actualTransition > 0) {
            if (distStart < actualTransition) {
              const t = distStart / actualTransition;
              envelope = t * t * (3 - 2 * t);
            } else if (distEnd < actualTransition) {
              const t = distEnd / actualTransition;
              envelope = t * t * (3 - 2 * t);
            }
          } else {
            envelope = 0;
          }
          
          x += Math.sin((prog * waveHeight / waveLength * Math.PI * 2) + phase) * maxAmplitude * envelope;
        }
        pts.push({ x, y });
      }
    }
    
    if (pts.length > 0 && pts[pts.length - 1].y < height - pad) {
      pts.push({ x: midX, y: height - pad });
    }

    if (pts.length === 0) return "";

    const lefts = [];
    const rights =[];

    for (let i = 0; i < pts.length; i++) {
      const p = pts[i];
      let dx, dy;
      
      if (i === 0) {
        dx = pts[1].x - pts[0].x;
        dy = pts[1].y - pts[0].y;
      } else if (i === pts.length - 1) {
        dx = pts[i].x - pts[i-1].x;
        dy = pts[i].y - pts[i-1].y;
      } else {
        dx = pts[i+1].x - pts[i-1].x;
        dy = pts[i+1].y - pts[i-1].y;
      }
      
      const len = Math.sqrt(dx * dx + dy * dy);
      const nx = len === 0 ? 1 : dy / len;
      const ny = len === 0 ? 0 : -dx / len;
      
      rights.push({ x: p.x + R * nx, y: p.y + R * ny });
      lefts.push({ x: p.x - R * nx, y: p.y - R * ny });
    }

    let d = `M ${lefts[0].x.toFixed(2)} ${lefts[0].y.toFixed(2)} `;
    d += `A ${R} ${R} 0 0 1 ${rights[0].x.toFixed(2)} ${rights[0].y.toFixed(2)} `;
    
    for (let i = 1; i < rights.length; i++) {
      d += `L ${rights[i].x.toFixed(2)} ${rights[i].y.toFixed(2)} `;
    }
    
    const lastIdx = pts.length - 1;
    d += `A ${R} ${R} 0 0 1 ${lefts[lastIdx].x.toFixed(2)} ${lefts[lastIdx].y.toFixed(2)} `;
    
    for (let i = lefts.length - 2; i >= 0; i--) {
      d += `L ${lefts[i].x.toFixed(2)} ${lefts[i].y.toFixed(2)} `;
    }
    
    d += "Z";
    return d;
  });
</script>

<div class="control-container">
  <div class="progress-track" bind:clientHeight={height}>
    {#if height > 0}
      <svg xmlns="http://www.w3.org/2000/svg">
        <defs>
          <clipPath id="wave-fill-clip">
            <rect 
              x="-5" 
              y="0"
              width="34" 
              height={height * (progress / 100)} 
            />
          </clipPath>
        </defs>
        <path d={pathD} class="wave-bg" />
        <path d={pathD} class="wave-fill" clip-path="url(#wave-fill-clip)" />
      </svg>
    {/if}
  </div>
  <div class="buttons">
    <button class="v-btn-icon control-btn-lesser" onclick={prev} title="Previous">
      <img src="/icons/outlined/24px/skip_previous.svg" alt="" class="rotated-icon" />
    </button>
    <button class="v-btn-icon control-btn" onclick={togglePlay} title="Toggle Play">
      <img src={isPlaying ? "/icons/outlined/24px/pause.svg" : "/icons/outlined/24px/play_arrow.svg"} alt="" />
    </button>
    <button class="v-btn-icon control-btn-lesser" onclick={next} title="Next">
      <img src="/icons/outlined/24px/skip_next.svg" alt="" class="rotated-icon" />
    </button>
  </div>
</div>

<style>
  .control-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    height: 100%;
    width: 100%;
  }

  .progress-track {
    width: 24px;
    flex: 1;
    position: relative;
    min-height: 60px;
    margin: 0 0 0 0;
    margin: 20px 0 4px 0;
  }

  svg {
    display: block;
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    overflow: visible;
  }

  .wave-bg {
    fill: oklch(100% 0 0 / 0.074);
    stroke: none;
  }

  .wave-fill {
    fill: oklch(100% 0 0 / 0.54);
    stroke: none;
  }

  .buttons {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .control-btn {
    width: 40px;
    height: 40px;
    margin: 0 2px;
    border-radius: 20px;
    flex-shrink: 0;
  }

  .control-btn img {
    width: 22px;
    height: 22px;
  }

  .control-btn-lesser {
    width: 36px;
    height: 36px;
    margin: 0 4px;
    border-radius: 20px;
    flex-shrink: 0;
  }

  .control-btn-lesser img {
    width: 18px;
    height: 18px;
  }

  .rotated-icon {
    transform: rotate(90deg);
  }
</style>
