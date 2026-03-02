<script>
  import { player } from "../player.svelte.js";

  let currentIndex = $derived.by(() => {
    const idx = player.queue.findIndex(item => item.file === player.currentFile);
    return idx !== -1 ? String(idx + 1) : "0";
  });

  let totalQueue = $derived(String(player.queue.length));
</script>

<div class="hud-corner-unit">
  <div class="hud-group">
    <span class="hud-label">IDX</span>
    <span class="hud-data">{currentIndex}</span>
    <span class="hud-sep">/</span>
    <span class="hud-data">{totalQueue}</span>
  </div>
</div>

<style>
  .hud-corner-unit {
    padding: 0 20px;
    height: 100%;
    display: flex;
    align-items: center;
  }

  .hud-group {
    display: flex;
    align-items: baseline;
    gap: 6px;
    font-family: var(--font-mono);
    color: #fff;
    font-size: 14px;
  }

  .hud-label {
    opacity: 0.5;
    font-size: 12px;
    margin-right: 4px;
  }

  .hud-sep {
    opacity: 0.3;
  }
</style>
