<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let currentFile = $derived(player.currentFile);
  let activeId = $derived(player.currentAlbumId);
  
  let fullAlbum = $derived(activeId ? library.fullAlbumCache[activeId] : null);
  let currentTrackFull = $derived(fullAlbum?.tracks?.find(t => t.info?.track_library_path === currentFile) || null);

  let lyricsText = $state("");
  let isLoading = $state(false);
  
  let isInstrumental = $derived(currentTrackFull?.tags?.INSTRUMENTAL === true);

  async function fetchLyrics(trackFull) {
    if (!trackFull) {
      lyricsText = "";
      return;
    }

    if (trackFull.tags?.INSTRUMENTAL === true) {
      lyricsText = "";
      isLoading = false;
      return;
    }

    if (trackFull.info?.lyrics_path) {
      isLoading = true;
      try {
          const encodedId = encodeURIComponent(activeId);
          const pathPart = trackFull.info.lyrics_path; 
          const url = `/api/assets/lyrics/${encodedId}/${pathPart}`;
          
          const res = await fetch(url);
          if (res.ok) {
              lyricsText = await res.text();
              isLoading = false;
              return;
          }
      } catch (e) {}
    }
    
    if (trackFull.tags && trackFull.tags.LYRICS) {
      lyricsText = trackFull.tags.LYRICS;
    } else {
      lyricsText = "";
    }
    
    isLoading = false;
  }

  $effect(() => {
    fetchLyrics(currentTrackFull);
  });
</script>

<div class="lyrics-container">
  {#if isLoading}
    <div class="status-msg">Loading...</div>
  {:else if isInstrumental}
    <div class="instrumental-msg">[INSTRUMENTAL]</div>
  {:else if lyricsText}
    <div class="lyrics-content">
      {#each lyricsText.split(/\r?\n/) as line}
        <p class="lyric-line">{line}</p>
      {/each}
    </div>
  {:else}
    <div class="status-msg">
      No lyrics available.
    </div>
  {/if}
</div>

<style>
  .lyrics-container {
    width: 100%;
    height: 100%;
    overflow-y: auto;
    background-color: transparent;
    box-sizing: border-box;
    display: flex;
  }

  .lyrics-container::-webkit-scrollbar {
    width: 0px;
  }

  .lyrics-content {
    font-family: var(--font-stack);
    font-size: 15px;
    line-height: 1.08;
    color: var(--text-main);
    text-align: center;
    text-align: left;
    margin: -6px auto;
    width: 100%;
  }

  .lyric-line {
    margin: 6px 0;
    min-height: 0.8em;
    text-wrap: balance;
  }

  .status-msg {
    margin: auto;
    color: oklch(100% 0 0 / 0.8);
    font-family: var(--font-mono);
    font-size: 14px;
    font-style: italic;
  }

  .instrumental-msg {
    font-family: var(--font-stack);
    font-family: var(--font-mono);
    font-size: 15px;
    line-height: 1.2;
    color: oklch(100% 0 0);
    margin: 0 auto;
    margin: auto;
    text-align: center;
  }
</style>
