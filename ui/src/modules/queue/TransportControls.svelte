<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let isPlaying = $derived(player.state === "play");
  
  let currentMeta = $derived(player.currentFile ? library.getTrackByPath(player.currentFile) : null);
  let title = $derived(currentMeta?.TITLE || player.title || "");
  let artist = $derived(currentMeta?.ARTIST || player.artist || "");
  
  async function togglePlay() {
    try { await fetch('/api/toggle-pause', { method: 'POST' }); } catch(e) {}
  }
  
  async function next() {
    try { await fetch('/api/next', { method: 'POST' }); } catch(e) {}
  }
  
  async function prev() {
    try { await fetch('/api/prev', { method: 'POST' }); } catch(e) {}
  }
</script>

<div class="transport-controls">
  <div class="meta-stack">
    <div class="artist" title={artist}>{artist}</div>
    <div class="title" title={title}>{title}</div>
  </div>

  <div class="control-row">
    <button class="ctrl-btn secondary" onclick={prev}>
      <img src="/icons/24px/skip_previous.svg" alt="Previous" />
    </button>
    <button class="ctrl-btn primary" onclick={togglePlay}>
      <img 
        src={isPlaying ? "/icons/24px/pause.svg" : "/icons/24px/play_arrow.svg"} 
        alt={isPlaying ? "Pause" : "Play"} 
      />
    </button>
    <button class="ctrl-btn secondary" onclick={next}>
      <img src="/icons/24px/skip_next.svg" alt="Next" />
    </button>
  </div>
</div>

<style>
  .transport-controls {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
    padding-top: 24px;
    box-sizing: border-box;
  }

  .meta-stack {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    text-align: center;
    width: 100%;
    overflow: hidden;
  }

  .artist {
    font-size: 16px;
    font-weight: 500;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .title {
    font-size: 14px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .control-row {
    display: flex;
    align-items: center;
    gap: 24px;
  }

  .ctrl-btn {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition: opacity 0.2s;
    opacity: 0.8;
  }

  .ctrl-btn:hover {
    opacity: 1;
  }

  .ctrl-btn img {
    width: 28px;
    height: 28px;
  }

  .ctrl-btn.primary img {
    width: 48px;
    height: 48px;
  }
</style>
