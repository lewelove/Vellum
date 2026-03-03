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
        // Path might contain slashes, but they shouldn't be encoded individually if part of path structure
        // However, the router expects {*path}, so we pass the relative path string
        const pathPart = meta.lyrics_path; 
        const url = `/api/assets/lyrics/${encodedId}/${pathPart}`;
        
        const res = await fetch(url);
        if (res.ok) {
            lyricsText = await res.text();
        } else {
            lyricsText = "";
        }
    } catch (e) {
        console.error("Lyrics fetch failed", e);
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
      {lyricsText}
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
    padding: 24px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
  }

  .lyrics-container::-webkit-scrollbar {
    width: 0px;
  }

  .lyrics-content {
    white-space: pre-wrap;
    font-family: var(--font-stack);
    font-size: 16px;
    line-height: 1.6;
    color: var(--text-main);
    text-align: center;
    margin: auto;
    max-width: 600px;
  }

  .status-msg {
    margin: auto;
    color: var(--text-muted);
    font-size: 14px;
    font-style: italic;
  }
</style>
