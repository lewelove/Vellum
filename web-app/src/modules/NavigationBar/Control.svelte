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
      phase -= dt * 0.0; 
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
    const pad = 4; 
    const straightLen = 16;
    const waveLength = 24;
    const maxAmplitude = 2;
    const transitionLen = 100;
    
    let d = `M ${midX} ${pad}`;
    
    if (height <= straightLen * 2 + pad * 2) {
      d += ` L ${midX} ${height - pad}`;
      return d;
    }
    
    d += ` L ${midX} ${straightLen + pad}`;
    
    const waveStart = straightLen + pad;
    const waveEnd = height - straightLen - pad;
    const waveHeight = waveEnd - waveStart;
    
    const points = Math.ceil(waveHeight / 2);
    for (let i = 1; i <= points; i++) {
      const y = waveStart + (i / points) * waveHeight;
      const prog = (y - waveStart) / waveHeight;
      
      let envelope = 1;
      const distStart = y - waveStart;
      const distEnd = waveEnd - y;
      
      if (distStart < transitionLen) {
        envelope = Math.sin((distStart / transitionLen) * (Math.PI / 2));
      } else if (distEnd < transitionLen) {
        envelope = Math.sin((distEnd / transitionLen) * (Math.PI / 2));
      }
      
      const x = midX + Math.sin((prog * waveHeight / waveLength * Math.PI * 2) + phase) * maxAmplitude * envelope;
      d += ` L ${x.toFixed(2)} ${y.toFixed(2)}`;
    }
    
    d += ` L ${midX} ${height - pad}`;
    return d;
  });
</script>

<div class="control-container">
  <div class="progress-track" bind:clientHeight={height}>
    {#if height > 0}
      <svg width="24" {height} xmlns="http://www.w3.org/2000/svg">
        <defs>
          <clipPath id="wave-fill-clip">
            <rect 
              x="0" 
              y="0"
              width="24" 
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
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 60px;
    overflow: hidden;
  }

  svg {
    display: block;
    overflow: visible;
  }

  .wave-bg {
    fill: none;
    stroke: oklch(100% 0 0 / 0.074);
    stroke-width: 4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .wave-fill {
    fill: none;
    stroke: oklch(100% 0 0 / 0.15);
    stroke: oklch(100% 0 0 / 0.54);
    stroke-width: 4;
    stroke-linecap: round;
    stroke-linejoin: round;
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
