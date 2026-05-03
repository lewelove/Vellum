<script lang="ts">
  import { onMount } from "svelte";
  import { player } from "../player.svelte.ts";
  import { library } from "../../library.svelte.ts";

  let activeId = $derived(player.currentAlbumId);

  $effect(() => {
    if (activeId) {
      library.ensureFullAlbum(activeId);
    }
  });

  function stripSlashes(str: string | null) {
    return (str || "").replace(/^\/+/, "");
  }

  let fullAlbum = $derived(activeId ? library.fullAlbumCache[activeId] : null);
  
  let currentTrackFull = $derived(
    (fullAlbum?.tracks && player.currentFile)
      ? fullAlbum.tracks.find((t: any) => stripSlashes(t.info?.track_library_path) === stripSlashes(player.currentFile))
      : null
  );

  let title = $derived(currentTrackFull?.TITLE || player.title || "");
  let artist = $derived(currentTrackFull?.ARTIST || player.artist || "");

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
  let progress = $derived(duration > 0 ? (Math.floor(tickingElapsed) / duration) * 100 : 0);

  function formatTime(totalSeconds: number) {
    const s = Math.floor(totalSeconds || 0);
    const m = Math.floor(s / 60);
    const rs = s % 60;
    const pad = (n: number) => String(n).padStart(2, '0');
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
      <span class="v-mono time">{formatTime(tickingElapsed)}  ∕  {formatTime(duration)}</span>
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
    height: 74px;
    padding: 16px 24px;
    box-sizing: border-box;
    border-radius: 0 0 24px 24px;
    margin-top: 0px;
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
    font-size: 15px;
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

