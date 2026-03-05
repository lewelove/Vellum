<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let tickingElapsed = $state(0);
  let duration = $derived(player.duration || 0);
  let progress = $derived(duration > 0 ? (tickingElapsed / duration) * 100 : 0);

  let isPlaying = $derived(player.state === "play");
  let currentMeta = $derived(player.currentFile ? library.getTrackByPath(player.currentFile) : null);
  let title = $derived(currentMeta?.TITLE || player.title || "");
  let artist = $derived(currentMeta?.ARTIST || player.artist || "");

  function formatTime(totalSeconds) {
    const s = Math.floor(totalSeconds || 0);
    const m = Math.floor(s / 60);
    const rs = s % 60;
    const pad = (n) => String(n).padStart(2, '0');
    return `${m}:${pad(rs)}`;
  }

  function tick() {
    if (player.state === "play") {
      const delta = (performance.now() - player.lastUpdated) / 1000;
      tickingElapsed = Math.min(player.elapsed + delta, player.duration);
    } else {
      tickingElapsed = player.elapsed;
    }
    requestAnimationFrame(tick);
  }

  async function togglePlay() {
    try { await fetch('/api/toggle-pause', { method: 'POST' }); } catch(e) {}
  }
  
  async function next() {
    try { await fetch('/api/next', { method: 'POST' }); } catch(e) {}
  }
  
  async function prev() {
    try { await fetch('/api/prev', { method: 'POST' }); } catch(e) {}
  }

  onMount(() => {
    const raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="unified-progress-bar">
  <!-- Interactive buttons -->
  <div class="transport-group">
    <button class="ctrl-btn" onclick={prev}>
      <img src="/icons/24px/skip_previous.svg" alt="Prev" />
    </button>
    <button class="ctrl-btn main" onclick={togglePlay}>
      <img src={isPlaying ? "/icons/24px/pause.svg" : "/icons/24px/play_arrow.svg"} alt="Toggle" />
    </button>
    <button class="ctrl-btn" onclick={next}>
      <img src="/icons/24px/skip_next.svg" alt="Next" />
    </button>
  </div>

  <!-- Non-interactive metadata -->
  <div class="metadata-group">
    <span class="artist">{artist}</span>
    <span class="separator">—</span>
    <span class="title">{title}</span>
  </div>

  <!-- Non-interactive visual progress -->
  <div class="slider-group">
    <span class="time">{formatTime(tickingElapsed)}</span>
    <div class="track-container">
      <div class="progress-track">
        <div class="progress-fill" style="width: {progress}%"></div>
      </div>
    </div>
    <span class="time">{formatTime(duration)}</span>
  </div>
</div>

<style>
  .unified-progress-bar {
    width: 100%;
    height: 64px;
    display: flex;
    align-items: center;
    padding: 0 24px;
    box-sizing: border-box;
    background-color: var(--background-drawer);
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    gap: 32px;
  }

  .transport-group {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .ctrl-btn {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  .ctrl-btn:hover {
    opacity: 1;
  }

  .ctrl-btn img {
    width: 20px;
    height: 20px;
  }

  .ctrl-btn.main img {
    width: 28px;
    height: 28px;
  }

  .metadata-group {
    display: flex;
    align-items: center;
    gap: 8px;
    white-space: nowrap;
    overflow: hidden;
    min-width: 150px;
    max-width: 400px;
    font-size: 14px;
    pointer-events: none; /* Solely visual */
  }

  .artist {
    color: var(--text-main);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .separator {
    color: var(--text-muted);
    opacity: 0.5;
    flex-shrink: 0;
  }

  .title {
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .slider-group {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 16px;
    min-width: 0;
    pointer-events: none; /* Solely visual */
  }

  .time {
    font-family: var(--font-mono);
    font-size: 13px;
    color: #888888;
    font-feature-settings: "tnum";
    min-width: 38px;
    text-align: center;
  }

  .track-container {
    flex: 1;
    height: 32px;
    display: flex;
    align-items: center;
  }

  .progress-track {
    position: relative;
    width: 100%;
    height: 3px;
    background-color: rgba(255, 255, 255, 0.06);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background-color: var(--text-main);
    border-radius: 2px;
  }
</style>
