<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let currentFile = $derived(player.currentFile);
  let lyricsText = $state("");
  let isLoading = $state(false);
  
  let currentMeta = $derived(currentFile ? library.getTrackByPath(currentFile) : null);

  async function fetchLyrics(meta) {
    if (!meta || !meta.lyrics_path) {
      lyricsText = "";
      return;
    }

    isLoading = true;
    try {
        const encodedId = encodeURIComponent(meta.albumId);
        const pathPart = meta.lyrics_path; 
        const url = `/api/assets/lyrics/${encodedId}/${pathPart}`;
        
        const res = await fetch(url);
        if (res.ok) {
            lyricsText = await res.text();
        } else {
            lyricsText = "";
        }
    } catch (e) {
        lyricsText = "";
    } finally {
        isLoading = false;
    }
  }

  $effect(() => {
    fetchLyrics(currentMeta);
  });
</script>

<div class="lyrics-container">
  {#if isLoading}
    <div class="status-msg">Loading...</div>
  {:else if lyricsText}
    <div class="lyrics-content">
      {#each lyricsText.split(/\r?\n/) as line}
        <p class="lyric-line">{line}</p>
      {/each}
    </div>
  {:else}
    <div class="status-msg">
      {#if currentMeta && !currentMeta.lyrics_path}
        No lyrics file linked.
      {:else}
        No lyrics available.
      {/if}
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
    line-height: 1.3;
    color: var(--text-main);
    text-align: left;
    margin: 0 auto;
    width: 100%;
    max-width: 300px;
  }

  .lyric-line {
    margin: 5px 0;
    min-height: 1.0em;
    text-wrap: balance;
  }

  .status-msg {
    margin: auto;
    color: var(--text-muted);
    font-size: 14px;
    font-style: italic;
  }
</style>
