<script>
  let { tracks = [], totalDiscs = "1", onplay } = $props();

  let selectedIndex = $state(-1);
  let multiDisc = $derived(parseInt(totalDiscs) > 1);

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

  function handlePlayDisc(discNumber) {
    const firstIndex = tracks.findIndex(t => t.DISCNUMBER === discNumber);
    if (firstIndex !== -1 && onplay) {
      onplay(firstIndex);
    }
  }

  function handleKeydown(e, index) {
    if (e.key === 'Enter') {
      handlePlay(index);
    }
  }
</script>

<div class="tracks-list">
  {#each tracks as track, i}
    {#if multiDisc && (i === 0 || track.DISCNUMBER !== tracks[i-1].DISCNUMBER)}
      {#if i > 0}
        <div class="disc-separator"></div>
      {/if}
      <div class="disc-header-row">
        <span class="disc-label">Disc {track.DISCNUMBER}</span>
        <button 
          class="disc-play-btn" 
          onclick={() => handlePlayDisc(track.DISCNUMBER)}
          title="Play Disc {track.DISCNUMBER}"
        >
          <img src="/material/play_circle_24dp_666666.svg" alt="Play Disc" />
        </button>
      </div>
    {/if}

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
  }

  .track-row + .track-row {
    margin-top: 4px;
  }

  .disc-separator {
    height: 1px;
    background-color: rgba(255, 255, 255, 0.05);
    margin: 16px 0;
    width: 100%;
  }

  .disc-header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    user-select: none;
    margin-bottom: 8px;
  }

  .disc-label {
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-size: 14px;
    font-weight: 600;
    color: #666;
    background-color: rgba(255, 255, 255, 0.00);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    height: 32px;
    box-sizing: border-box;
  }

  .disc-play-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 32px;
    cursor: pointer;
    background-color: rgba(255, 255, 255, 0.01);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    transition: background-color 0.1s;
    box-sizing: border-box;
  }

  .disc-play-btn:hover {
    background-color: rgba(255, 255, 255, 0.05);
  }

  .disc-play-btn img {
    width: 24px;
    height: 24px;
  }

  .track-row {
    display: flex;
    align-items: center;
    padding: 8px 0px;
    font-size: 16px;
    color: #ffffff;
    cursor: default;
    outline: none;
    user-select: none;
    background-color: transparent;
    contain: layout;
    box-sizing: border-box;
    border-radius: 8px;
    transition: background-color 0.1s ease, border-color 0.1s ease;
  }

  .track-row:hover {
    background-color: #2b2b2b;
    border-color: rgba(255, 255, 255, 0.05);
  }

  .track-row.selected {
    background-color: #333333;
    border-color: rgba(255, 255, 255, 0.1);
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
