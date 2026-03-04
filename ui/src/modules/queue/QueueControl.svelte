<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let tickingElapsed = $state(0);
  
  let isPlaying = $derived(player.state === "play");
  let duration = $derived(player.duration || 0);
  let progress = $derived(duration > 0 ? (tickingElapsed / duration) * 100 : 0);
  
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

<div class="queue-control">
  <div class="left-group">
    <div class="controls">
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
    
    <div class="separator"></div>

    <div class="meta">
      <span class="meta-text">
        <span class="artist">{artist}</span>
        {#if artist && title}<span class="dash">-</span>{/if}
        <span class="title">{title}</span>
      </span>
    </div>
  </div>

  <div class="progress-section">
    <div class="progress-track">
      <div class="progress-fill" style="width: {progress}%"></div>
      <div class="progress-knob" style="left: {progress}%"></div>
    </div>
  </div>

  <div class="right-group">
    <span class="time">{formatTime(tickingElapsed)} / {formatTime(duration)}</span>
    
    <!-- <button class="tool-btn"> -->
    <!--   <img src="/icons/24px/volume_up.svg" alt="Volume" /> -->
    <!-- </button> -->
  </div>
</div>

<style>
  .queue-control {
    position: relative;
    height: 64px;
    background-color: var(--background-drawer);
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 24px;
    z-index: 500;
    pointer-events: auto;
    overflow: hidden;
    box-shadow: var(--modal-shadow);
    margin: 12px 36px 12px 36px;
    flex-shrink: 0;
    gap: 24px;
  }

  .left-group {
    display: flex;
    align-items: center;
    gap: 24px;
    flex-shrink: 0;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 16px;
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
    width: 20px;
    height: 20px;
  }

  .ctrl-btn.primary img {
    width: 24px;
    height: 24px;
  }

  .separator {
    width: 1px;
    height: 24px;
    background-color: rgba(255, 255, 255, 0.1);
  }

  .meta {
    display: flex;
    align-items: center;
    overflow: hidden;
  }

  .meta-text {
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .artist {
    color: var(--text-main);
    font-weight: 500;
  }

  .dash {
    color: var(--text-muted);
    margin: 0 6px;
  }

  .title {
    color: var(--text-muted);
  }

  .progress-section {
    flex: 1;
    display: flex;
    align-items: center;
    padding: 0 12px;
  }

  .progress-track {
    position: relative;
    width: 100%;
    height: 2px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
  }

  .progress-fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background-color: var(--text-main);
    border-radius: 2px;
  }

  .progress-knob {
    position: absolute;
    top: 50%;
    width: 8px;
    height: 8px;
    background-color: var(--text-main);
    border-radius: 50%;
    transform: translate(-50%, -50%);
    opacity: 0;
    transition: opacity 0.2s;
  }

  .queue-control:hover .progress-knob {
    opacity: 1;
  }

  .right-group {
    display: flex;
    align-items: center;
    gap: 24px;
    flex-shrink: 0;
  }

  .time {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-muted);
    font-feature-settings: "tnum";
  }

  .tool-btn {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.6;
    transition: opacity 0.2s;
    padding: 0;
  }

  .tool-btn:hover {
    opacity: 1;
  }

  .tool-btn img {
    width: 20px;
    height: 20px;
  }
</style>
