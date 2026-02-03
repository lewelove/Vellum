<script>
  let { tracks = [], onplay } = $props();

  let selectedIndex = $state(-1);

  function formatDuration(str) {
    if (!str) return "";
    return str.replace(/^(00:)+/, "").replace(/^0/, "");
  }

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

<div class="tracks-list">
  {#each tracks as track, i}
    <div 
      class="track-row" 
      class:selected={selectedIndex === i}
      tabindex="0"
      onclick={() => handleSelect(i)}
      ondblclick={() => handlePlay(i)}
      onkeydown={(e) => handleKeydown(e, i)}
      role="button"
      aria-label="Track {track.TRACKNUMBER}: {track.TITLE}"
    >
      <span class="track-index">{track.TRACKNUMBER}</span>
      <div class="track-body">
        <span class="track-title">{track.TITLE}</span>
        <span class="track-artist">{track.ARTIST === track.ALBUMARTIST ? '' : track.ARTIST}</span>
      </div>
      <span class="track-meta">{formatDuration(track.track_duration_time)}</span>
    </div>
  {/each}
</div>

<style>
  .tracks-list {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .track-row {
    display: flex;
    align-items: center;
    padding: 10px 12px;
    font-size: 14px;
    color: var(--text-main);
    cursor: default;
    outline: none;
    user-select: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  }

  .track-row:last-child {
    border-bottom: none;
  }

  .track-row:hover {
    background-color: rgba(255, 255, 255, 0.03);
  }

  .track-row.selected {
    background-color: rgba(255, 255, 255, 0.06);
  }

  .track-index {
    flex: 0 0 40px;
    opacity: 0.4;
    font-family: var(--font-mono);
    font-size: 13px;
  }

  .track-body {
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
  }

  .track-artist {
    font-size: 12px;
    opacity: 0.5;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .track-meta {
    flex-shrink: 0;
    opacity: 0.4;
    font-family: var(--font-mono);
    font-size: 13px;
  }
</style>
