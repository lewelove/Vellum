<script>
  let { 
    tracks = [], 
    totalDiscs = "1", 
    albumArtist = "",
    onplay, 
    onplaydisc 
  } = $props();

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

  function getDiscDuration(discNumber) {
    const totalMs = tracks
      .filter(t => t.DISCNUMBER === discNumber)
      .reduce((acc, t) => acc + (parseInt(t.track_duration) || 0), 0);
    return formatMs(totalMs);
  }

  function handleSelect(index) {
    selectedIndex = index;
  }

  function handlePlay(index) {
    if (onplay) onplay(index);
  }

  function handlePlayDisc(discNumber) {
    if (onplaydisc) {
      onplaydisc(discNumber);
    } else {
      const firstIndex = tracks.findIndex(t => t.DISCNUMBER === discNumber);
      if (firstIndex !== -1 && onplay) {
        onplay(firstIndex);
      }
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
        
        <div class="disc-header-right">
          <span class="v-mono disc-duration-label">{getDiscDuration(track.DISCNUMBER)}</span>
          <button 
            class="v-btn-icon disc-play-btn" 
            onclick={() => handlePlayDisc(track.DISCNUMBER)}
            title="Play Disc {track.DISCNUMBER}"
          >
            <img src="/icons/20px/play_arrow.svg" alt="Play Disc" />
          </button>
        </div>
      </div>
    {/if}

    <div 
      class="v-track-row track-row" 
      class:active={selectedIndex === i}
      tabindex="0"
      onclick={() => handleSelect(i)}
      ondblclick={() => handlePlay(i)}
      onkeydown={(e) => handleKeydown(e, i)}
      role="button"
      aria-label="Track {track.TRACKNUMBER}: {track.TITLE}"
    >
      <span class="v-mono track-index">{track.TRACKNUMBER}</span>
      <div class="track-body">
        <span class="v-truncate track-title">{track.TITLE}</span>
        {#if track.ARTIST && albumArtist && track.ARTIST.toLowerCase() !== albumArtist.toLowerCase()}
          <span class="v-truncate track-artist">{track.ARTIST}</span>
        {/if}
      </div>
      <span class="v-mono track-meta">{formatDuration(track.track_duration_time)}</span>
    </div>
  {/each}
</div>

<style>
  .tracks-list {
    display: flex;
    flex-direction: column;
    width: 100%;
    background-color: transparent;
    box-sizing: border-box;
  }

  .track-row + .track-row {
    margin-top: 4px;
  }

  .disc-separator {
    height: 1px;
    background-color: rgba(255, 255, 255, 0.05);
    margin: 12px 0;
    width: 100%;
  }

  .disc-header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    user-select: none;
    margin-bottom: 8px;
  }

  .disc-header-right {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .disc-label, .disc-duration-label {
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-size: 12px;
    font-weight: 600;
    color: #666;
    background-color: rgba(255, 255, 255, 0.00);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    height: 24px;
    box-sizing: border-box;
  }

  .disc-duration-label {
    font-weight: 400;
  }

  .disc-play-btn {
    width: 36px;
    height: 24px;
    border-radius: 8px;
  }

  .disc-play-btn img {
    width: 18px;
    height: 18px;
  }

  .track-index {
    flex: 0 0 44px;
    text-align: center;
    color: #888888;
    font-size: 12px;
    line-height: 18px;
  }

  .track-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    margin-right: 16px;
    line-height: 18px;
  }

  .track-title {
    word-break: keep-all;
    overflow-wrap: break-word;
  }

  .track-artist {
    font-size: 13px;
    color: #999999;
    margin-top: 4px;
  }

  .track-meta {
    color: #888888;
    padding-right: 18px;
    text-align: right;
    font-size: 13px;
    min-width: 44px;
  }
</style>
