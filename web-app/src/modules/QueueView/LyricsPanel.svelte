<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let currentFile = $derived(player.currentFile);
  let lyricsText = $state("");
  let isLoading = $state(false);
  
  let currentMeta = $derived(currentFile ? library.getTrackByPath(currentFile) : null);
  let isInstrumental = $derived(currentMeta?.tags?.INSTRUMENTAL === true);

  async function fetchLyrics(meta) {
    if (!meta) {
      lyricsText = "";
      return;
    }

    if (meta.tags?.INSTRUMENTAL === true) {
      lyricsText = "";
      isLoading = false;
      return;
    }

    if (meta.lyrics_path) {
      isLoading = true;
      try {
          const encodedId = encodeURIComponent(meta.albumId);
          const pathPart = meta.lyrics_path; 
          const url = `/api/assets/lyrics/${encodedId}/${pathPart}`;
          
          const res = await fetch(url);
          if (res.ok) {
              lyricsText = await res.text();
              isLoading = false;
              return;
          }
      } catch (e) {
      }
    }
    
    if (meta.tags && meta.tags.LYRICS) {
      lyricsText = meta.tags.LYRICS;
    } else {
      lyricsText = "";
    }
    
    isLoading = false;
  }

  $effect(() => {
    fetchLyrics(currentMeta);
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
    line-height: 1.2;
    color: var(--text-main);
    text-align: left;
    margin: 0 auto;
    width: 100%;
    max-width: 300px;
    max-width: 280px;
  }

  .lyric-line {
    margin: 6px 0;
    min-height: 0.4em;
    text-wrap: balance;
  }

  .status-msg {
    margin: auto;
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 14px;
    font-style: italic;
  }

  .instrumental-msg {
    font-family: var(--font-stack);
    font-family: var(--font-mono);
    font-size: 15px;
    line-height: 1.2;
    color: oklch(100% 0 0 / 0.8);
    margin: auto;
    margin: 0 auto;
    text-align: center;
  }
</style>
