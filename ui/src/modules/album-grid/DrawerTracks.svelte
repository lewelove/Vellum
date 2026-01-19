<script>
  let { tracks = [], cols = 1 } = $props();

  // Distribution Engine
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
          <!-- Segment 1: Gutter (Track Number) -->
          <span class="track-index">{track.number}</span>
          
          <!-- Segment 2: Title Block -->
          <span class="track-title">{track.title}</span>
          
          <!-- Segment 3: Meta (Duration) -->
          <span class="track-meta">{track.duration}</span>
        </div>
      {/each}
    </div>
  {/each}
</div>

<style>
  .tracks-grid {
    display: grid;
    gap: 40px; /* Horizontal gap between columns */
    width: 100%;
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

  /* Segment 1: Fixed Gutter */
  .track-index {
    flex: 0 0 28px;
    text-align: right;
    margin-right: 12px;
    color: var(--text-muted);
    font-family: monospace;
  }

  /* Segment 2: Fluid Title */
  .track-title {
    flex: 1;
    text-overflow: ellipsis;
    overflow: hidden;
    font-weight: 400;
    /* font-family: monospace; */
  }

  /* Segment 3: Meta Segment */
  .track-meta {
    flex: 0 0 45px;
    text-align: right;
    color: var(--text-muted);
    font-size: 0.9em;
    padding-left: 8px;
  }
</style>
