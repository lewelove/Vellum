<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let tickingElapsed = $state(0);

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

  function formatSeconds(totalSeconds) {
    const s = Math.floor(totalSeconds || 0);
    const m = Math.floor(s / 60);
    const rs = s % 60;
    const pad = (n) => String(n).padStart(2, '0');
    return `${m}:${pad(rs)}`;
  }

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

  function formatMs(ms) {
    if (!ms) return "0:00";
    const totalSeconds = Math.floor(ms / 1000);
    const h = Math.floor(totalSeconds / 3600);
    const m = Math.floor((totalSeconds % 3600) / 60);
    const s = totalSeconds % 60;
    
    const pad = (num) => String(num).padStart(2, '0');

    if (h > 0) {
      return `${h}:${pad(m)}:${pad(s)}`;
    }
    return `${m}:${pad(s)}`;
  }

  function getDiscDuration(tracks, discNumber) {
    const totalMs = tracks
      .filter(t => t.discNo === discNumber)
      .reduce((acc, t) => {
        const meta = library.getTrackByPath(t.file);
        return acc + (parseInt(meta?.track_duration) || 0);
      }, 0);
    return formatMs(totalMs);
  }

  let mappedTracks = $derived(player.queue.map(item => {
    const meta = library.getTrackByPath(item.file);
    const albumId = meta?.albumId || null;
    
    const title = meta ? meta.TITLE : (item.title || item.file);
    const artist = meta ? meta.ARTIST : (item.artist || "");

    return {
      id: item.id,
      file: item.file,
      isPlaying: player.currentFile === item.file,
      trackNo: meta ? meta.TRACKNUMBER : "#",
      discNo: meta ? meta.DISCNUMBER : "1",
      duration: meta ? meta.track_duration_time : "",
      title,
      artist,
      albumId
    };
  }));

  let groupedQueue = $derived.by(() => {
    const groups =[];
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

      {@const isMultiDiscAlbum = group.albumMeta && parseInt(group.albumMeta.total_discs || "1") > 1}

      {#each group.tracks as track, i (track.id)}
        {@const showDiscHeader = isMultiDiscAlbum && (i === 0 || track.discNo !== group.tracks[i-1].discNo)}
        
        {#if showDiscHeader}
          {#if i > 0}
            <div class="disc-separator"></div>
          {/if}
          <div class="disc-header-row" class:first-disc={i === 0}>
            <span class="disc-label">Disc {track.discNo}</span>
            <div class="disc-header-right">
              <span class="disc-duration-label">{getDiscDuration(group.tracks, track.discNo)}</span>
            </div>
          </div>
        {/if}

        <div class="track-row" class:active={track.isPlaying}>
          <span class="track-index">{track.trackNo}</span>
          <div class="track-body">
            <span class="track-title">{track.title}</span>
            {#if track.artist && group.albumMeta && track.artist.toLowerCase() !== group.albumMeta.ALBUMARTIST.toLowerCase()}
              <span class="track-artist">{track.artist}</span>
            {/if}
          </div>
          <span class="track-meta">
            {#if track.isPlaying}
              {formatSeconds(tickingElapsed)} / {formatDuration(track.duration)}
            {:else}
              {formatDuration(track.duration)}
            {/if}
          </span>
        </div>
      {/each}
    {/each}
  </div>
</div>

<style>
  .header-row,
  .disc-header-row,
  .track-row,
  .header-album,
  .header-artist,
  .header-meta,
  .disc-label,
  .disc-duration-label,
  .track-index,
  .track-title,
  .track-artist,
  .track-meta {
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .header-album,
  .track-row,
  .track-title {
    color: #ffffff;
  }

  .header-artist,
  .track-artist {
    color: rgba(255, 255, 255, 0.8);
  }

  .header-meta,
  .disc-label,
  .disc-duration-label,
  .track-index,
  .track-meta {
    color: rgba(255, 255, 255, 0.6);
  }

  .tracks-list-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    background-color: transparent;
    min-height: 0;
    overflow: hidden;
  }

  .tracks-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 0 12px 0;
    min-height: 0;
  }

  .tracks-list::-webkit-scrollbar {
    width: 0px;
  }

  .album-group-header {
    padding: 0px 0px 12px 0px;
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
    font-size: 16px;
    white-space: nowrap;
    overflow: hidden;
    word-break: keep-all;
    overflow-wrap: break-word;
  }

  .header-artist {
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    word-break: keep-all;
    overflow-wrap: break-word;
  }

  .header-meta {
    font-size: 13px;
    white-space: nowrap;
    margin-left: 8px;
    font-feature-settings: "tnum";
  }

  .disc-separator {
    height: 1px;
    background-color: rgba(255, 255, 255, 0.05);
    margin: 12px 20px;
  }

  .disc-header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 0px;
    margin-bottom: 8px;
    margin-top: 8px;
  }

  .disc-header-row.first-disc {
    margin-top: 0px;
  }

  .disc-header-right {
    display: flex;
    align-items: center;
  }

  .disc-label,
  .disc-duration-label {
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-size: 12px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    height: 24px;
    box-sizing: border-box;
  }

  .disc-label {
    font-weight: 600;
  }

  .disc-duration-label {
    font-feature-settings: "tnum";
    font-weight: 400;
  }

  .track-row {
    position: relative;
    display: flex;
    align-items: flex-start;
    padding: 4px 0px;
    font-size: 14px;
    cursor: default;
    user-select: none;
    background-color: transparent;
    /* background-color: rgba(36, 36, 36, 0.16); */
    border-radius: 8px;
    border: 1px solid transparent;
    /* border: 1px solid rgba(255, 255, 255, 0.1); */
    margin: 0 0px;
    /* transition: background-color 0.1s ease; */
    overflow: hidden;
  }

  .track-row + .track-row {
    margin-top: 4px;
  }

  .track-row:hover {
    background-color: rgba(255, 255, 255, 0.01);
    border-color: rgba(255, 255, 255, 0.04);
  }

  .track-row.active {
    background-color: rgba(255, 255, 255, 0.02);
    border-color: rgba(255, 255, 255, 0.05);
  }

  .track-index {
    font-feature-settings: "tnum";
    position: relative;
    z-index: 1;
    flex: 0 0 44px;
    text-align: center;
    font-size: 12px;
    line-height: 18px;
  }

  .track-body {
    position: relative;
    z-index: 1;
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    margin-right: 16px;
    min-width: 0;
  }

  .track-title {
    font-size: 14px;
    line-height: 18px;
    word-break: keep-all;
    overflow-wrap: break-word;
  }

  .track-artist {
    font-size: 13px;
    margin-top: 4px;
    line-height: 16px;
    word-break: keep-all;
    overflow-wrap: break-word;
  }

  .track-meta {
    position: relative;
    z-index: 1;
    text-align: right;
    font-size: 13px;
    font-feature-settings: "tnum";
    padding-right: 18px;
    min-width: 44px;
    line-height: 18px;
  }
</style>
