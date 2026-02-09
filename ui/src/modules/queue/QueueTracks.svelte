<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  // Helper for time formatting matching ModalDrawerTracks
  function formatDuration(str) {
    if (!str) return "0:00";
    
    let parts = str.split(':');
    
    while (parts.length > 2 && parseInt(parts[0]) === 0) {
      parts.shift();
    }
    
    if (parts[0].length > 1 && parts[0].startsWith('0')) {
      parts[0] = parts[0].substring(1);
    }
    
    return parts.join(':');
  }

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
      discNo: meta ? meta.DISCNUMBER : "1",
      duration: meta ? meta.track_duration_time : "",
      title,
      artist,
      showArtist,
      albumId,
      albumArtist
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
      <!-- Album Header -->
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

      {@const isMultiDiscAlbum = group.albumMeta && parseInt(group.albumMeta.TOTALDISCS || "1") > 1}

      {#each group.tracks as track, i (track.id)}
        {@const showDiscHeader = isMultiDiscAlbum && (i === 0 || track.discNo !== group.tracks[i-1].discNo)}
        
        {#if showDiscHeader}
          {#if i > 0}
            <div class="disc-separator"></div>
          {/if}
          <div class="disc-header-row" class:first-disc={i === 0}>
            <span class="disc-label">Disc {track.discNo}</span>
          </div>
        {/if}

        <div class="track-row" class:active={track.isPlaying}>
          {#if track.isPlaying}
            <div class="row-progress" style="width: {playbackPercent}%"></div>
          {/if}
          
          <span class="track-index">{track.trackNo}</span>
          <div class="track-body">
            <span class="track-title">{track.title}</span>
            {#if track.showArtist}
              <span class="track-artist">{track.artist}</span>
            {/if}
          </div>
          <span class="track-meta">{formatDuration(track.duration)}</span>
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

  .tracks-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 0 16px 0;
  }

  /* --- Album Header --- */
  .album-group-header {
    padding: 12px 20px 12px 20px;
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
    box-shadow: 0 2px 4px rgba(0,0,0,0.2);
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

  .header-album {
    font-size: 15px;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
  }

  .header-artist {
    font-size: 14px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .header-meta {
    font-size: 13px;
    color: var(--text-muted);
    opacity: 0.7;
    white-space: nowrap;
    margin-left: 8px;
    font-feature-settings: "tnum";
  }

  /* --- Disc Headers --- */
  .disc-separator {
    height: 1px;
    background-color: rgba(255, 255, 255, 0.05);
    margin: 12px 20px;
  }

  .disc-header-row {
    display: flex;
    align-items: center;
    padding: 0 16px;
    margin-bottom: 8px;
    margin-top: 8px;
  }

  .disc-header-row.first-disc {
    margin-top: 0px;
  }

  .disc-label {
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-size: 12px;
    font-weight: 600;
    color: #666;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    height: 24px;
    box-sizing: border-box;
  }

  /* --- Track Row (Matches ModalDrawerTracks) --- */
  .track-row {
    position: relative;
    display: flex;
    align-items: center;
    padding: 6px 0px;
    font-size: 14px;
    color: #ffffff;
    cursor: default;
    user-select: none;
    background-color: transparent;
    border-radius: 8px;
    margin: 0 8px;
    transition: background-color 0.1s ease;
    overflow: hidden;
  }

  .track-row + .track-row {
    margin-top: 4px;
  }

  .track-row:hover {
    background-color: rgba(255, 255, 255, 0.03);
  }

  .track-row.active {
    background-color: rgba(255, 255, 255, 0.04);
  }

  .row-progress {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    background-color: rgba(255, 255, 255, 0.06);
    z-index: 0;
    pointer-events: none;
  }

  .track-index {
    font-feature-settings: "tnum";
    position: relative;
    z-index: 1;
    flex: 0 0 44px;
    text-align: center;
    color: #888888;
    font-size: 12px;
  }

  .track-body {
    position: relative;
    z-index: 1;
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    margin-right: 16px;
  }

  .track-title {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 14px;
    color: #ffffff;
  }

  .track-artist {
    font-size: 13px;
    color: #999999;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .track-meta {
    position: relative;
    z-index: 1;
    color: #888888;
    text-align: right;
    font-size: 13px;
    font-feature-settings: "tnum";
    padding-right: 18px;
    min-width: 44px;
  }
</style>
