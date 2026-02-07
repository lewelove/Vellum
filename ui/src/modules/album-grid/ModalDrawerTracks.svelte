<script>
  let { tracks = [], onplay } = $props();

  let selectedIndex = $state(-1);

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
    background-color: #242424;
    box-sizing: border-box;
    gap: 4px;
  }

  .track-row {
    display: flex;
    align-items: center;
    padding: 8px 0px;
    font-size: 15px;
    color: #ffffff;
    cursor: default;
    outline: none;
    user-select: none;
    background-color: transparent;
    contain: layout;
    border-radius: 10px;
    transition: background-color 0.1s ease;
  }

  .track-row:hover {
    background-color: #2b2b2b;
  }

  .track-row.selected {
    background-color: #333333;
  }

  .track-index {
    flex: 0 0 44px;
    text-align: center;
    color: #888888;
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
    font-size: 13px;
    color: #999999;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .track-meta {
    color: #888888;
    padding-right: 18px;
    text-align: right;
    font-size: 13px;
  }
</style>
