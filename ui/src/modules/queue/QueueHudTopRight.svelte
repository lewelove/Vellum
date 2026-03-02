<script>
  import { player } from "../player.svelte.js";

  let currentIndex = $derived.by(() => {
    const idx = player.queue.findIndex(item => item.file === player.currentFile);
    return idx !== -1 ? String(idx + 1) : "0";
  });

  let totalQueue = $derived(String(player.queue.length));
</script>

<div class="hud-corner-unit">
  <div class="vga-group">
    <span class="vga-label">IDX</span>
    <span class="vga-data">{currentIndex}</span>
    <span class="vga-sep">/</span>
    <span class="vga-data">{totalQueue}</span>
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

  .vga-label {
    opacity: 0.5;
    font-size: 12px;
    margin-right: 4px;
  }

  .vga-sep {
    opacity: 0.3;
  }
</style>
