<script>
  let { tracks = [], cols = 1, onplay } = $props();

  let selectedIndex = $state(-1);

  let columnData = $derived.by(() => {
    const rowsPerCol = Math.ceil(tracks.length / cols);
    const result = [];
    for (let i = 0; i < cols; i++) {
      const start = i * rowsPerCol;
      const end = (i + 1) * rowsPerCol;
      result.push(
        tracks.slice(start, end).map((track, localIdx) => ({
          ...track,
          globalIdx: start + localIdx
        }))
      );
    }
    return result;
  });

  function handleSelect(index) {
    selectedIndex = index;
  }

  function handlePlay(index) {
    if (onplay) onplay(index);
  }

  function handleKeydown(e, index) {
    if (e.key === 'Enter') {
      handlePlay(index);
    }
  }
</script>

<div class="tracks-grid" style="grid-template-columns: repeat({cols}, 1fr);">
  {#each columnData as column}
    <div class="track-column">
      {#each column as track}
        <div 
          class="track-row" 
          class:selected={selectedIndex === track.globalIdx}
          tabindex="0"
          onclick={() => handleSelect(track.globalIdx)}
          ondblclick={() => handlePlay(track.globalIdx)}
          onkeydown={(e) => handleKeydown(e, track.globalIdx)}
          role="button"
          aria-label="Track {track.TRACKNUMBER}: {track.TITLE}"
        >
          <span class="track-index">{track.TRACKNUMBER}</span>
          <span class="track-title">{track.TITLE}</span>
          <span class="track-meta">{track.track_duration_time}</span>
        </div>
      {/each}
    </div>
  {/each}
</div>

<style>
  .tracks-grid {
    display: grid;
    /* gap: 12px; */
    width: 100%;
    
    -webkit-font-smoothing: subpixel-antialiased;
    text-rendering: optimizeLegibility;
  }

  .track-column {
    display: flex;
    flex-direction: column;
  }

  .track-row {
    display: flex;
    align-items: center;
    padding-right: 12px;
    margin-right: 12px;
    height: var(--drawer-track-y);
    font-size: var(--drawer-font-size-track);
    color: var(--text-main);
    /* border-bottom: 1px solid rgba(255, 255, 255, 0.05); */
    white-space: nowrap;
    overflow: hidden;
    cursor: default;
    outline: none;
    user-select: none;
  }

  .track-row:hover {
    background-color: rgba(255, 255, 255, 0.03);
  }

  .track-row.selected {
    background-color: rgba(255, 255, 255, 0.07);
  }

  .track-row.selected .track-title {
    color: var(--text-main);
  }

  .track-index {
    flex: 0 0 48px;
    text-align: center;
    /* margin-right: px; */
    color: var(--text-muted);
    font-family: monospace;
  }

  .track-title {
    flex: 1;
    text-overflow: ellipsis;
    overflow: hidden;
    font-weight: 400;
  }

  .track-meta {
    flex: 0 0 0px;
    text-align: right;
    color: var(--text-muted);
    font-size: 13px;
    font-family: monospace;
    padding-left: 12px;
  }
</style>
