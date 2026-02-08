<script>
  import { fade } from "svelte/transition";
  import { playAlbum, queueAlbum, openAlbumFolder } from "../../api.js";
  import { library } from "../../library.svelte.js";
  import ModalDrawerCover from "./ModalDrawerCover.svelte";
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

  async function handleOpenFolder() {
    try {
      await openAlbumFolder(album.id);
    } catch (err) {
      console.error("Failed to open folder:", err);
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
            <ModalDrawerCover 
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
            {#if album.DATE}
              <span class="original-date">{album.DATE}</span>
              <span class="meta-sep">•</span>
            {/if}
            <span class="album-duration">{album.album_duration_time || ""}</span>
            {#if album.MEDIA}
              <span class="meta-sep">•</span>
              <span class="media">{album.MEDIA}</span>
            {/if}
          </div>
        </div>

        <div class="footer-container">
          <div class="footer-line">
            <p class="album-comment">{album.COMMENT || ""}</p>
          </div>
        </div>
      </div>

      <div class="column-right">
        <div class="button-bar">
          <div class="bar-group">
            <button class="icon-btn" onclick={handleOpenFolder} title="Open Local Folder">
              <img src="/material/folder_FFFFFF.svg" alt="Open"/>
            </button>
          </div>

          <div class="bar-group right">
            <button class="icon-btn" onclick={handleQueue} title="Add Album to Queue">
              <img src="/material/playlist_add_FFFFFF.svg" alt="" />
            </button>
            <button class="icon-btn" onclick={handlePlay} title="Play Album">
              <img src="/material/playlist_play_FFFFFF.svg" alt="" />
            </button>
          </div>
        </div>
        <div class="tracks-scroll-area">
          <div class="scroll-fade-overlay-top"></div>
          <ModalDrawerTracks 
            tracks={album.tracks} 
            totalDiscs={album.TOTALDISCS} 
            onplay={handlePlayTrack} 
          />
          <div class="scroll-fade-overlay-bottom"></div>
        </div>
      </div>

    </div>
  </div>
</div>

<style>
  .button-bar {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 10px;
    padding-bottom: 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    width: 100%;
  }

  .bar-group {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .bar-group img {
    opacity: 0.8;
  }

  .bar-group.right {
    margin-left: auto;
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-chassis {
    width: 75vw;
    height: 90vh;
    background-color: #242424;
    box-shadow: 0 0 64px rgba(0, 0, 0, 0.1), 0 0 48px rgba(0, 0, 0, 0.3), 0 0 32px rgba(0, 0, 0, 0.5);
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
    background-color: #1f1f1f;
    border-right: 1px solid rgba(255, 255, 255, 0.05);
    min-width: 0;
    min-height: 0;
    box-sizing: border-box;
  }

  .cover-container {
    width: 100%;
    flex-shrink: 0;
    background-color: transparent;
    overflow: visible;
  }

  .meta-container {
    margin-top: 24px;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .album-title {
    margin: 0;
    font-size: 26px;
    font-weight: 400;
    color: #ffffff;
    word-wrap: break-word;
  }

  .album-artist {
    margin: 12px 0 0 0;
    font-size: 21px;
    font-weight: 400;
    color: #d3d3d3;
    word-wrap: break-word;
  }

  .meta-row {
    display: flex;
    align-items: center;
    min-height: 24px;
    margin-top: 24px;
    font-size: 16px;
    color: #888888;
    gap: 12px;
    white-space: nowrap;
  }

  .meta-sep {
    color: #444444;
  }

  .footer-container {
    margin-top: auto;
    padding-top: 24px;
    min-width: 0;
  }

  .footer-line {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    min-width: 0;
  }

  .album-comment {
    margin: 0;
    font-size: 16px;
    color: #999999;
    font-style: italic;
    word-wrap: break-word;
    flex: 1;
  }

  .column-right {
    display: flex;
    flex-direction: column;
    padding: 32px;
    min-width: 0;
    min-height: 0;
    height: 100%;
    box-sizing: border-box;
    background-color: #242424;
  }

  .icon-btn {
    width: 40px;
    height: 40px;
    background-color: rgba(255, 255, 255, 0.01);
    border: 1px solid rgba(255, 255, 255, 0.05);
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-radius: 8px;
    box-shadow: var(--button-shadow);
    transition: background-color 0.1s, transform 0.1s;
  }

  .icon-btn:hover {
    background-color: rgba(255, 255, 255, 0.05);
  }

  .icon-btn img {
    width: 24px;
    height: 24px;
    pointer-events: none;
  }

  .tracks-scroll-area {
    position: relative;
    flex: 1;
    overflow-y: scroll;
    min-height: 0;
    background-color: #242424;
    transform: translateZ(0);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .scroll-fade-overlay-top {
    position: sticky;
    top: 0;
    left: 0;
    right: 0;
    height: 16px;
    background: linear-gradient(to bottom, #242424 0%, transparent 100%);
    z-index: 10;
    pointer-events: none;
  }

  .scroll-fade-overlay-bottom {
    position: sticky;
    bottom: 0;
    left: 0;
    right: 0;
    height: 16px;
    background: linear-gradient(to top, #242424 0%, transparent 100%);
    z-index: 10;
    pointer-events: none;
  }

  .tracks-scroll-area::-webkit-scrollbar {
    width: 0px;
  }
</style>
