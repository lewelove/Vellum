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
    <ProgressBar />
  </div>
</div>

<style>
  .control-panel {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    height: 54px;
    padding: 0 32px;
    box-sizing: border-box;
    border-radius: 12px;
    margin-top: 16px;
    flex-shrink: 0;
  }

  .left-zone {
    display: flex;
    align-items: center;
  }

  .right-zone {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    justify-content: center;
    width: 450px;
    max-width: 60%;
    gap: 8px;
  }

  .metadata {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    width: 100%;
    justify-content: flex-end;
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

  /* --- Scoped Global Overrides for Subcomponents --- */

  :global(.control-panel .controls) {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  :global(.control-panel .ctrl-btn) {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  :global(.control-panel .ctrl-btn:hover) {
    opacity: 1;
  }

  :global(.control-panel .ctrl-btn img) {
    width: 24px;
    height: 24px;
  }

  :global(.control-panel .ctrl-btn.main img) {
    width: 32px;
    height: 32px;
  }

  :global(.control-panel .progress-wrapper) {
    display: flex;
    align-items: center;
    width: 100%;
    gap: 12px;
  }

  :global(.control-panel .time) {
    font-size: 12px;
    color: var(--text-muted);
    min-width: 40px;
  }

  :global(.control-panel .time:first-child) {
    text-align: right;
  }

  :global(.control-panel .time:last-child) {
    text-align: left;
  }

  :global(.control-panel .track-container) {
    flex: 1;
    height: 24px;
    display: flex;
    align-items: center;
  }

  :global(.control-panel .progress-track) {
    position: relative;
    width: 100%;
    height: 4px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }

  :global(.control-panel .progress-fill) {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background-color: var(--text-main);
    border-radius: 2px;
  }
</style>
