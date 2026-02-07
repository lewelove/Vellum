<script>
  import { fade } from "svelte/transition";
  import { playAlbum, queueAlbum } from "../../api.js";
  import { library } from "../../library.svelte.js";
  import SmartImage from "./SmartImage.svelte";
  import ModalDrawerTracks from "./ModalDrawerTracks.svelte";

  let { album, onclose } = $props();

  let leftColumnWidth = $state(0);
  let coverUrl = $derived(library.getAlbumCoverUrl(album.id));

  async function handlePlay() {
    try {
      await playAlbum(album.id);
    } catch (err) {
      console.error("Failed to play album:", err);
    }
  }

  async function handleQueue() {
    try {
      await queueAlbum(album.id);
    } catch (err) {
      console.error("Failed to queue album:", err);
    }
  }

  async function handlePlayTrack(index) {
    try {
      await playAlbum(album.id, index);
    } catch (err) {
      console.error("Failed to play track:", err);
    }
  }

  function handleBackdropClick(e) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }
</script>

<div 
  class="modal-backdrop" 
  onclick={handleBackdropClick} 
  role="presentation"
  transition:fade={{ duration: 100 }}
>
  <div class="modal-chassis">
    <div class="modal-content">
      
      <div class="column-left" bind:clientWidth={leftColumnWidth}>
        <div class="cover-container" style="height: {leftColumnWidth - 64}px;">
          {#if leftColumnWidth > 0}
            <SmartImage 
              src={coverUrl} 
              width={leftColumnWidth - 64} 
              height={leftColumnWidth - 64} 
            />
          {/if}
        </div>

        <div class="meta-container">
          <h2 class="album-title">{album.title}</h2>
          <h3 class="album-artist">{album.artist}</h3>
          <div class="meta-row">
            {#if album.ORIGINAL_DATE}
              <span class="original-date">{album.ORIGINAL_DATE}</span>
            {/if}
          </div>
        </div>

        <div class="footer-container">
          <div class="footer-line">
            <p class="album-comment">{album.COMMENT || ""}</p>
            <span class="album-duration">{album.album_duration_time || ""}</span>
          </div>
        </div>
      </div>

      <div class="column-right">
        <div class="button-bar">
          <button class="icon-btn" onclick={handleQueue} title="Add Album to Queue">
            <img src="/material/playlist_add_FFFFFF.svg" alt="" />
          </button>
          <button class="icon-btn" onclick={handlePlay} title="Play Album">
            <img src="/material/playlist_play_FFFFFF.svg" alt="" />
          </button>
        </div>
        <div class="tracks-scroll-area">
          <ModalDrawerTracks tracks={album.tracks} onplay={handlePlayTrack} />
        </div>
      </div>

    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.2);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-chassis {
    width: 75vw;
    height: 80vh;
    background-color: var(--background-drawer);
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
    border-radius: 12px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .modal-content {
    display: grid;
    grid-template-columns: 42% 58%;
    grid-template-rows: 100%;
    height: 100%;
    width: 100%;
    min-height: 0;
  }

  .column-left {
    display: flex;
    flex-direction: column;
    padding: 32px;
    background-color: rgba(0, 0, 0, 0.15);
    border-right: 1px solid rgba(255, 255, 255, 0.05);
    min-width: 0;
    min-height: 0;
    box-sizing: border-box;
  }

  .cover-container {
    width: 100%;
    flex-shrink: 0;
    box-shadow: 0 0 32px rgba(0, 0, 0, 0.4);
    background-color: #222;
    overflow: hidden;
  }

  .meta-container {
    margin-top: 20px;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .album-title {
    margin: 0;
    font-size: 26px;
    font-weight: 400;
    color: var(--text-main);
    word-wrap: break-word;
  }

  .album-artist {
    margin: 10px 0 0 0;
    font-size: 23px;
    font-weight: 400;
    color: var(--text-muted);
    line-height: 1.2;
    word-wrap: break-word;
  }

  .meta-row {
    display: flex;
    align-items: center;
    min-height: 24px;
    margin-top: 8px;
  }

  .original-date {
    font-size: 16px;
    color: var(--text-muted);
    opacity: 0.8;
  }

  .footer-container {
    margin-top: auto;
    padding-top: 24px;
    min-width: 0;
  }

  .footer-line {
    display: flex;
    justify-content: space-between;
    align-items: bottom;
    min-height: 24px;
    gap: 16px;
  }

  .album-comment {
    margin: 0;
    font-size: 16px;
    color: var(--text-muted);
    font-style: italic;
    line-height: 1.2;
    word-wrap: break-word;
    opacity: 0.7;
    flex: 1;
  }

  .album-duration {
    font-size: 16px;
    color: var(--text-muted);
    opacity: 0.7;
    white-space: nowrap;
  }

  .column-right {
    display: flex;
    flex-direction: column;
    padding: 32px;
    min-width: 0;
    min-height: 0;
    height: 100%;
    box-sizing: border-box;
    background-color: var(--background-drawer);
  }

  .button-bar {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 10px;
    margin-bottom: 16px;
    height: 32px;
  }

  .icon-btn {
    width: 40px;
    height: 40px;
    background-color: rgba(255, 255, 255, 0.05);
    border: none;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-radius: 8px;
    transition: background-color 0.1s;
  }

  .icon-btn:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  .icon-btn img {
    width: 24px;
    height: 24px;
    pointer-events: none;
  }

  .tracks-scroll-area {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }
</style>
