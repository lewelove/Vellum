<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  const pad = (num) => String(num).padStart(2, '0');

  const formatTime = (seconds) => {
    const s = Math.floor(seconds || 0);
    const m = Math.floor(s / 60);
    const rs = s % 60;
    return `${pad(m)}:${pad(rs)}`;
  };

  let currentIndex = $derived.by(() => {
    const idx = player.queue.findIndex(item => item.file === player.currentFile);
    return idx !== -1 ? String(idx + 1) : "0";
  });

  let totalQueue = $derived(String(player.queue.length));
  
  let timeElapsed = $derived(formatTime(player.elapsed));
  let timeTotal = $derived(formatTime(player.duration));

  let mappedTracks = $derived(player.queue.map(item => {
    const meta = library.getTrackByPath(item.file);
    const title = meta ? meta.TITLE : (item.title || item.file);
    const artist = meta ? meta.ARTIST : (item.artist || "");
    const albumArtist = meta ? meta.ALBUMARTIST : (item.albumartist || "");

    const showArtist = artist && 
                       albumArtist && 
                       artist.toLowerCase() !== albumArtist.toLowerCase();

    return {
      id: item.id,
      file: item.file,
      isPlaying: player.currentFile === item.file,
      trackNo: meta ? meta.TRACKNUMBER : "#",
      title,
      artist,
      showArtist
    };
  }));
</script>

<div class="queue-view-wrapper">
  <div class="vga-recessed-well">
      <div class="vga-layer active">
        <div class="vga-line">
          <span class="vga-label">trk:</span>
          <span class="vga-data">{currentIndex}</span>
          <span class="vga-separator">/</span>
          <span class="vga-data">{totalQueue}</span>
        </div>
        <div class="vga-line">
          <span class="vga-data">{timeElapsed}</span>
          <span class="vga-separator">/</span>
          <span class="vga-data">{timeTotal}</span>
        </div>
      </div>

  </div>

  <div class="tracks-list">
    {#each mappedTracks as track (track.id)}
      <div class="queue-row" class:active={track.isPlaying}>
        <span class="col-index">{track.trackNo}</span>
        <div class="col-info">
            <span class="q-title" title={track.title}>{track.title}</span>
            {#if track.showArtist}
                <span class="q-artist" title={track.artist}>{track.artist}</span>
            {/if}
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .queue-view-wrapper {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    background-color: transparent;
  }

  .vga-recessed-well {
    padding: 8px 16px;
    background-color: transparent;
    border-bottom: 1px solid var(--border-muted);
    display: flex;
    justify-content: flex-end;
    align-items: center;
    overflow: hidden;
  }

  .vga-screen {
    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: 1fr;
    position: relative;
  }

  .vga-layer {
    grid-area: 1 / 1;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }

  .vga-layer.active {
    z-index: 1;
    text-shadow: 0 0 4px rgba(255, 255, 255, 0.4);
  }

  .vga-line {
    display: flex;
    align-items: baseline;
    justify-content: flex-end;
    font-family: monospace;
    color: #fff;
    line-height: 1;
    letter-spacing: 0.05em;
  }

  .vga-label {
    font-size: 16px;
    opacity: 1;
    font-weight: 400;
  }

  .vga-data {
    font-size: 16px;
    font-weight: 100;
  }

  .vga-separator {
    font-size: 16px;
    font-weight: 400;
  }

  .tracks-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .queue-row {
    display: flex;
    align-items: center;
    padding: 8px 12px 8px 12px;
    color: var(--text-muted);
  }

  .queue-row:hover {
    background-color: rgba(255, 255, 255, 0.02);
    color: var(--text-main);
  }

  .queue-row.active {
    background-color: rgba(255, 255, 255, 0.04);
    color: var(--text-main);
  }

  .col-index {
    flex: 0 0 38px;
    text-align: center;
    font-size: 14px;
    font-family: var(--font-mono);
    opacity: 0.5;
  }

  .col-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    justify-content: center;
  }

  .q-title {
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
    color: inherit;
  }

  .q-artist {
    font-size: 13px;
    opacity: 0.6;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
