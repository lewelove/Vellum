<script>
  import { player } from "../player.svelte.js";
</script>

<div class="hud-corner-unit">
  <div class="metadata-line">
    <span class="title-text">{player.title || "STOPPED"}</span>
    <span class="sep-text">::</span>
    <span class="artist-text">{player.artist || "---"}</span>
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
    font-family: var(--font-mono);
  }

  .title-text {
    color: #fff;
    font-size: 14px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .sep-text {
    color: #666;
    font-size: 12px;
  }

  .artist-text {
    color: #999;
    font-size: 13px;
    text-transform: uppercase;
  }
</style>
