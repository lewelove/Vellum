<script>
  import { player } from "../player.svelte.js";
  import NavTabs from "../navigation/NavTabs.svelte";
</script>

<div class="hud-corner-unit">
  <div class="hud-nav-group">
    <NavTabs />
  </div>

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
    gap: 20px;
  }

  .hud-nav-group {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .metadata-line {
    display: flex;
    align-items: baseline;
    gap: 10px;
    font-family: var(--font-mono);
    min-width: 0;
    overflow: hidden;
  }

  .title-text {
    color: #fff;
    font-size: 14px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sep-text {
    color: #666;
    font-size: 12px;
    flex-shrink: 0;
  }

  .artist-text {
    color: #999;
    font-size: 13px;
    text-transform: uppercase;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
