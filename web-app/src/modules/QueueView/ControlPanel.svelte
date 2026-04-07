<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let currentMeta = $derived(player.currentFile ? library.getTrackByPath(player.currentFile) : null);
  let title = $derived(currentMeta?.TITLE || player.title || "");
  let artist = $derived(currentMeta?.ARTIST || player.artist || "");

  let isPlaying = $derived(player.state === "play");

  async function togglePlay() { 
    try { await fetch('/api/toggle-pause', { method: 'POST' }); } catch(e) {} 
  }
  
  async function next() { 
    try { await fetch('/api/next', { method: 'POST' }); } catch(e) {} 
  }
  
  async function prev() { 
    try { await fetch('/api/prev', { method: 'POST' }); } catch(e) {} 
  }

  let tickingElapsed = $state(0);
  let duration = $derived(player.duration || 0);
  let progress = $derived(duration > 0 ? (tickingElapsed / duration) * 100 : 0);

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
      tickingElapsed = Math.min(player.elapsed + delta, player.duration || 0);
    } else {
      tickingElapsed = player.elapsed || 0;
    }
    requestAnimationFrame(tick);
  }

  onMount(() => {
    const raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="control-panel v-glass">
  <div class="left-zone">
    <button class="v-btn-icon control-btn" onclick={prev} title="Previous">
      <img src="/icons/outlined/24px/skip_previous.svg" alt="" />
    </button>
    <button class="v-btn-icon control-btn" onclick={togglePlay} title="Toggle Play">
      <img src={isPlaying ? "/icons/outlined/24px/pause.svg" : "/icons/outlined/24px/play_arrow.svg"} alt="" />
    </button>
    <button class="v-btn-icon control-btn" onclick={next} title="Next">
      <img src="/icons/outlined/24px/skip_next.svg" alt="" />
    </button>
  </div>
  
  <div class="right-zone">
    <div class="info-row">
      <div class="metadata">
        {#if artist || title}
          <span class="artist" title={artist}>{artist}</span>
          <span class="separator">—</span>
          <span class="title" title={title}>{title}</span>
        {/if}
      </div>
      <span class="v-mono time">{formatTime(tickingElapsed)} / {formatTime(duration)}</span>
    </div>
    <div class="progress-track">
      <div class="progress-fill" style="width: {progress}%"></div>
    </div>
  </div>
</div>

<style>
  .control-panel {
    display: flex;
    align-items: stretch;
    width: 100%;
    height: 64px;
    padding: 12px 24px;
    box-sizing: border-box;
    border-radius: 24px;
    margin-top: 16px;
    flex-shrink: 0;
    gap: 24px;
  }

  .left-zone {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .control-btn {
    width: 40px;
    height: 40px;
    border-radius: 20px;
    flex-shrink: 0;
  }

  .control-btn img {
    width: 22px;
    height: 22px;
  }

  .right-zone {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    min-width: 0;
    padding: 2px 0 6px 0;
    box-sizing: border-box;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    gap: 16px;
  }

  .metadata {
    display: flex;
    justify-content: flex-start;
    align-items: baseline;
    gap: 8px;
    font-size: 15px;
    white-space: nowrap;
    overflow: hidden;
    flex: 1;
  }

  .artist {
    color: var(--text-main);
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
  }

  .separator {
    color: var(--text-main);
    opacity: 0.5;
    flex-shrink: 0;
  }

  .title {
    color: var(--text-main);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .time {
    font-size: 14px;
    color: oklch(100% 0 0 / 0.8);
    flex-shrink: 0;
  }

  .progress-track {
    width: 100%;
    height: 4px;
    background-color: oklch(100% 0 0 / 0.1);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background-color: oklch(100% 0 0 / 0.7);
    border-radius: 1px;
  }
</style>
