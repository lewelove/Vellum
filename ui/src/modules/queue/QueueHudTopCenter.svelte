<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";

  let activeAlbum = $derived(library.albumCache.get(player.currentAlbumId));
  
  let displayString = $derived.by(() => {
    if (!activeAlbum) return "";
    const artist = activeAlbum.ALBUMARTIST || "Unknown Artist";
    const album = activeAlbum.ALBUM || "Unknown Album";
    return `${artist} : ${album}`;
  });
</script>

<div class="queue-hud-top-center">
  {#if displayString}
    <div class="metadata-wrapper">
      <span class="vga-line">{displayString}</span>
    </div>
  {/if}
</div>

<style>
  .queue-hud-top-center {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    overflow: hidden;
    padding: 0 20px;
  }

  .metadata-wrapper {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .vga-line {
    color: #fff;
    font-size: 16px !important;
  }
</style>
