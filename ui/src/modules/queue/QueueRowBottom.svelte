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

    if (h > 0) {
      return `${h}:${pad(m)}:${pad(rs)}`;
    }
    return `${m}:${pad(rs)}`;
  };

  let currentIndex = $derived.by(() => {
    const idx = player.queue.findIndex(item => item.file === player.currentFile);
    return idx !== -1 ? String(idx + 1) : "0";
  });

  let totalQueue = $derived(String(player.queue.length));
  let timeTotal = $derived(formatTime(player.duration));

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

<div class="queue-row-bottom">
  <div class="vga-readout">
    <div class="vga-group">
      <span class="vga-label">TRK:</span>
      <span class="vga-data">{currentIndex}</span>
      <span class="vga-sep">/</span>
      <span class="vga-data">{totalQueue}</span>
    </div>

    <div class="vga-divider"></div>

    <div class="vga-group">
      <span class="vga-data">{formatTime(tickingElapsed)}</span>
      <span class="vga-sep">/</span>
      <span class="vga-data">{timeTotal}</span>
    </div>
  </div>
</div>

<style>
  .queue-row-bottom {
    height: 36px;
    width: 100%;
    box-sizing: border-box;
    background-color: transparent;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 16px;
    flex-shrink: 0;
    box-shadow: var(--panel-shadow);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .vga-readout {
    display: flex;
    align-items: center;
    gap: 20px;
    text-shadow: 0 0 4px rgba(255, 255, 255, 0.4);
  }

  .vga-group {
    display: flex;
    align-items: baseline;
    gap: 6px;
    font-family: var(--font-mono);
    color: #fff;
    font-size: 16px;
    line-height: 1;
    letter-spacing: 0.05em;
  }

  .vga-label {
    opacity: 0.7;
  }

  .vga-sep {
    opacity: 0.5;
  }

  .vga-divider {
    width: 1px;
    height: 16px;
    background-color: var(--border-muted);
    opacity: 0.5;
  }
</style>
