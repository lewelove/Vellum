<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  const formatTime = (totalSeconds) => {
    const s = Math.floor(totalSeconds || 0);
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const rs = s % 60;
    
    const pad = (num) => String(num).padStart(2, '0');

    if (h > 0) {
      return `${h}:${pad(m)}:${pad(rs)}`;
    }
    return `${m}:${pad(rs)}`;
  };

  let currentIndex = $derived.by(() => {
    const idx = player.queue.findIndex(item => item.file === player.currentFile);
    return idx !== -1 ? String(idx + 1) : "0";
  });

  let totalQueue = $derived(String(player.queue.length));
  
  let timeElapsed = $derived(formatTime(player.elapsed));
  let timeTotal = $derived(formatTime(player.duration));

  // 1. Map individual tracks with metadata
  let mappedTracks = $derived(player.queue.map(item => {
    const meta = library.getTrackByPath(item.file);
    const albumId = meta?.album_id || null;
    
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
      showArtist,
      albumId
    };
  }));

  // 2. Group tracks by consecutive album
  let groupedQueue = $derived.by(() => {
    const groups = [];
    mappedTracks.forEach(track => {
      if (groups.length === 0 || groups[groups.length - 1].albumId !== track.albumId) {
        const albumMeta = library.albumCache.get(track.albumId);
        groups.push({
          albumId: track.albumId,
          albumMeta,
          tracks: [track]
        });
      } else {
        groups[groups.length - 1].tracks.push(track);
      }
    });
    return groups;
  });

  // 3. Playback percentage calculation
  let playbackPercent = $derived(
    player.duration > 0 ? (player.elapsed / player.duration) * 100 : 0
  );
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
    {#each groupedQueue as group}
      {#if group.albumMeta}
        <div class="album-group-header">
          <img 
            class="header-thumb" 
            src={library.getThumbnailUrl(group.albumMeta)} 
            alt="cover"
          />
          <div class="header-content">
            <div class="header-row">
              <span class="header-artist">{group.albumMeta.ALBUMARTIST}</span>
              <span class="header-meta">{group.albumMeta.ORIGINAL_YEAR || group.albumMeta.DATE?.substring(0,4)}</span>
            </div>
            <div class="header-row">
              <span class="header-album">{group.albumMeta.ALBUM}</span>
              <span class="header-meta">{group.albumMeta.album_duration_time}</span>
            </div>
          </div>
        </div>
      {/if}

      {#each group.tracks as track (track.id)}
        <div class="queue-row" class:active={track.isPlaying}>
          <span class="col-index">{track.trackNo}</span>
          <div class="col-info">
            <div class="q-title-wrapper">
              {#if track.isPlaying}
                <div class="q-title-progress" style="width: {playbackPercent}%"></div>
              {/if}
              <span class="q-title" title={track.title}>{track.title}</span>
            </div>
            {#if track.showArtist}
              <span class="q-artist" title={track.artist}>{track.artist}</span>
            {/if}
          </div>
        </div>
      {/each}
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

  .vga-layer {
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
    font-family: var(--font-mono);
    color: #fff;
    line-height: 1;
    letter-spacing: 0.05em;
  }

  .vga-label {
    font-size: 16px;
  }

  .vga-data {
    font-size: 16px;
    font-weight: 100;
  }

  .vga-separator {
    font-size: 16px;
  }

  /* --- New Album Group Header --- */
  .album-group-header {
    /* height: 48px; */
    padding: 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    box-sizing: border-box;
  }

  .header-thumb {
    width: 32px;
    height: 32px;
    object-fit: cover;
    background-color: #000;
  }

  .header-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-width: 0;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    line-height: 1.2;
  }

  .header-artist, .header-album {
    font-size: 16px;
    font-weight: 500;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-transform: uppercase;
  }

  .header-meta {
    font-size: 14px;
    color: var(--text-muted);
    opacity: 0.8;
    white-space: nowrap;
    margin-left: 8px;
  }

  .tracks-list {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }

  .queue-row {
    display: flex;
    align-items: center;
    padding: 6px 12px;
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
    font-size: 12px;
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

  /* --- Progress Bar Title --- */
  .q-title-wrapper {
    position: relative;
    display: inline-block;
    width: 100%;
    overflow: hidden;
  }

  .q-title-progress {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    background-color: rgba(255, 255, 255, 0.1);
    z-index: 0;
    pointer-events: none;
  }

  .q-title {
    position: relative;
    z-index: 1;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
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
