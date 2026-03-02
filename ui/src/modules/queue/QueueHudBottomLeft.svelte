<script>
  import { player } from "../player.svelte.js";
</script>

<div class="hud-corner-unit">
  <div class="metadata-line">
    <span class="vga-title">{player.title || "STOPPED"}</span>
    <span class="vga-sep">::</span>
    <span class="vga-artist">{player.artist || "---"}</span>
  </div>
</div>

<style>
  .hud-corner-unit {
    padding: 0 20px;
    height: 100%;
    display: flex;
    align-items: center;
  }

  .metadata-line {
    display: flex;
    align-items: baseline;
    gap: 10px;
    text-shadow: 0 0 4px rgba(255, 255, 255, 0.3);
    font-family: var(--font-mono);
  }

  .vga-title {
    color: #fff;
    font-size: 14px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .vga-sep {
    color: #666;
    font-size: 12px;
  }

  .vga-artist {
    color: #999;
    font-size: 13px;
    text-transform: uppercase;
  }
</style>
