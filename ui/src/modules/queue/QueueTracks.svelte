<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let mappedTracks = $derived(player.queue.map(item => {
    const meta = library.getTrackByPath(item.file);
    
    const title = meta ? meta.TITLE : (item.title || item.file);
    const artist = meta ? meta.ARTIST : (item.artist || "");
    const albumArtist = meta ? meta.ALBUMARTIST : (item.albumartist || "");

    // Only show the sub-label if artist exists and is different from the album artist
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

<div class="queue-tracks-container">
  <div class="tracks-header">
    <span class="header-label">Play Queue</span>
    <span class="count">{mappedTracks.length}</span>
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
  .queue-tracks-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    background-color: transparent;
  }

  .tracks-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 16px 16px 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 14px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.15em;
  }

  .count {
    opacity: 0.5;
    font-size: 14px;
  }

  .tracks-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .queue-row {
    display: flex;
    align-items: center;
    padding: 6px 12px 6px 6px;
    font-family: var(--font-stack);
    color: var(--text-muted);
    min-height: 44px;
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
    flex: 0 0 54px;
    text-align: center;
    font-size: 16px;
    font-family: monospace;
    opacity: 0.5;
    /* margin-right: 12px; */
    /* margin-left: 12px; */
  }

  .col-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    justify-content: center;
  }

  .q-title {
    font-size: 15px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
    color: inherit;
  }

  .q-artist {
    font-size: 14px;
    opacity: 0.6;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
