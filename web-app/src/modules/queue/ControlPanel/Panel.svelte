<script>
  import Controls from "./Controls.svelte";
  import ProgressBar from "./ProgressBar.svelte";
  import { player } from "../../player.svelte.js";
  import { library } from "../../../library.svelte.js";

  let currentMeta = $derived(player.currentFile ? library.getTrackByPath(player.currentFile) : null);
  let title = $derived(currentMeta?.TITLE || player.title || "");
  let artist = $derived(currentMeta?.ARTIST || player.artist || "");
</script>

<div class="control-panel v-glass">
  <div class="left-zone">
    <Controls />
  </div>
  
  <div class="right-zone">
    <div class="metadata">
      {#if artist || title}
        <span class="artist" title={artist}>{artist}</span>
        <span class="separator">—</span>
        <span class="title" title={title}>{title}</span>
      {/if}
    </div>
    <div class="telemetry">
      <ProgressBar />
    </div>
  </div>
</div>

<style>
  .control-panel {
    display: flex;
    align-items: center;
    width: 100%;
    height: 64px;
    padding: 12px 24px;
    box-sizing: border-box;
    border-radius: 12px;
    margin-top: 16px;
    flex-shrink: 0;
    gap: 24px;
  }

  .left-zone {
    flex-shrink: 0;
  }

  .right-zone {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-width: 0;
    gap: 5px;
  }

  .metadata {
    display: flex;
    justify-content: flex-start;
    gap: 8px;
    font-size: 15px;
    white-space: nowrap;
    overflow: hidden;
    width: 100%;
  }

  .artist {
    color: var(--text-main);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .separator {
    color: var(--text-muted);
    opacity: 0.5;
    flex-shrink: 0;
  }

  .title {
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .telemetry {
    width: 100%;
  }
</style>
