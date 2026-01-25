<script>
  let { tracks = [], cols = 1 } = $props();

  let columnData = $derived.by(() => {
    const rowsPerCol = Math.ceil(tracks.length / cols);
    const result = [];
    for (let i = 0; i < cols; i++) {
      result.push(tracks.slice(i * rowsPerCol, (i + 1) * rowsPerCol));
    }
    return result;
  });
</script>

<div class="tracks-grid" style="grid-template-columns: repeat({cols}, 1fr);">
  {#each columnData as column}
    <div class="track-column">
      {#each column as track}
        <div class="track-row">
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
    gap: 40px;
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
    height: var(--drawer-track-y);
    font-size: var(--drawer-font-size-track);
    color: var(--text-main);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    white-space: nowrap;
    overflow: hidden;
  }

  .track-index {
    flex: 0 0 28px;
    text-align: right;
    margin-right: 12px;
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
    flex: 0 0 45px;
    text-align: right;
    color: var(--text-muted);
    font-size: 0.9em;
    padding-left: 8px;
  }
</style>
