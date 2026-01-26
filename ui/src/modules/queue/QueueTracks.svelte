<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let { isVisible } = $props();

  let mappedTracks = $derived(player.queue.map(item => {
    // MPD 'file' matches 'track_library_path' in metadata
    const meta = library.getTrackByPath(item.file);
    return {
      id: item.id, // MPD pos/id
      file: item.file,
      isPlaying: player.currentFile === item.file,
      trackNo: meta ? meta.TRACKNUMBER : "#",
      title: meta ? meta.TITLE : (item.title || item.file),
      artist: meta ? meta.ARTIST : (item.artist || ""),
    };
  }));

  // Scroll current track into view logic could be added here
</script>

<div class="queue-tracks-container" class:visible={isVisible}>
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
            {#if track.artist}
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
    padding-top: 12px;
    box-sizing: border-box;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .queue-tracks-container.visible {
    opacity: 1;
  }

  .tracks-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px 12px 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 14px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .count {
    opacity: 0.5;
    font-size: 12px;
  }

  .tracks-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .queue-row {
    display: flex;
    align-items: center;
    padding: 6px 16px 6px 12px;
    font-family: var(--font-stack);
    color: var(--text-muted);
    border-left: 3px solid transparent;
  }

  .queue-row:hover {
    background-color: rgba(255, 255, 255, 0.03);
    color: var(--text-main);
  }

  .queue-row.active {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-main);
    border-left-color: var(--text-main);
  }

  .col-index {
    flex: 0 0 32px;
    text-align: right;
    font-size: 12px;
    font-family: monospace;
    opacity: 0.6;
    margin-right: 12px;
  }

  .col-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .q-title {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
  }

  .q-artist {
    font-size: 11px;
    opacity: 0.7;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
