<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

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

<div class="tracks-list-container">
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
              <span class="header-album">{group.albumMeta.ALBUM}</span>
              <span class="header-meta">{group.albumMeta.ORIGINAL_YEAR || group.albumMeta.DATE?.substring(0,4)}</span>
            </div>
            <div class="header-row">
              <span class="header-artist">{group.albumMeta.ALBUMARTIST}</span>
              <span class="header-meta">{group.albumMeta.album_duration_time}</span>
            </div>
          </div>
        </div>
      {/if}

      {#each group.tracks as track (track.id)}
        <div class="queue-row" class:active={track.isPlaying}>
          {#if track.isPlaying}
            <div class="row-progress" style="width: {playbackPercent}%"></div>
          {/if}
          
          <span class="col-index">{track.trackNo}</span>
          <div class="col-info">
            <span class="q-title" title={track.title}>{track.title}</span>
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
  .tracks-list-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    background-color: transparent;
  }

  .album-group-header {
    padding: 12px 16px 12px 20px;
    display: flex;
    align-items: center;
    gap: 12px;
    box-sizing: border-box;
  }

  .header-thumb {
    width: 40px;
    height: 40px;
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
  }

  .header-artist, .header-album {
    font-size: 16px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .header-album {
    font-size: 15px;
    color: var(--text-main);
  }

  .header-artist {
    font-size: 14px;
    color: var(--text-muted);
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
    position: relative;
    display: flex;
    align-items: center;
    padding: 8px 16px 8px 20px;
    color: var(--text-muted);
    overflow: hidden;
  }

  .queue-row:hover {
    background-color: rgba(255, 255, 255, 0.02);
    color: var(--text-main);
  }

  .queue-row.active {
    background-color: rgba(255, 255, 255, 0.04);
    color: var(--text-main);
  }

  .row-progress {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    background-color: rgba(255, 255, 255, 0.08);
    z-index: 0;
    pointer-events: none;
  }

  .col-index {
    position: relative;
    z-index: 1;
    flex: 0 0 24px;
    padding-right: 12px;
    text-align: center;
    font-size: 12px;
    font-family: var(--font-mono);
    opacity: 0.5;
  }

  .col-info {
    position: relative;
    z-index: 1;
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
