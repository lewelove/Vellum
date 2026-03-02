<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";

  let tickingElapsed = $state(0);

  const formatTime = (totalSeconds) => {
    const s = Math.floor(totalSeconds || 0);
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const rs = s % 60;
    const pad = (num) => String(num).padStart(2, '0');
    if (h > 0) return `${h}:${pad(m)}:${pad(rs)}`;
    return `${m}:${pad(rs)}`;
  };

  function tick() {
    if (player.state === "play") {
      const delta = (performance.now() - player.lastUpdated) / 1000;
      tickingElapsed = Math.min(player.elapsed + delta, player.duration);
    } else {
      tickingElapsed = player.elapsed;
    }
    requestAnimationFrame(tick);
  }

  onMount(() => {
    const raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="hud-corner-unit">
  <div class="vga-group">
    <span class="vga-data">{formatTime(tickingElapsed)}</span>
    <span class="vga-sep">/</span>
    <span class="vga-data">{formatTime(player.duration)}</span>
  </div>
</div>

<style>
  .hud-corner-unit {
    padding: 0 20px;
    height: 100%;
    display: flex;
    align-items: center;
  }

  .vga-group {
    display: flex;
    align-items: baseline;
    gap: 6px;
    font-family: var(--font-mono);
    color: #fff;
    font-size: 14px;
    text-shadow: 0 0 4px rgba(255, 255, 255, 0.4);
  }

  .vga-sep {
    opacity: 0.3;
  }
</style>
