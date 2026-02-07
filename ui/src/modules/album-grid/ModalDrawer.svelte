<script>
  import { fade } from "svelte/transition";
  import { playAlbum } from "../../api.js";
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
          
          <div class="actions-row">
            <button class="play-all-btn" onclick={handlePlay}>PLAY ALBUM</button>
          </div>
        </div>
      </div>

      <div class="column-right">
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
    width: 80vw;
    height: 85vh;
    /* max-width: 1400px; */
    background-color: var(--background-drawer);
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
    border-radius: 12px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .modal-content {
    display: grid;
    grid-template-columns: 45% 55%;
    height: 100%;
    width: 100%;
  }

  .column-left {
    display: flex;
    flex-direction: column;
    padding: 32px;
    background-color: rgba(0, 0, 0, 0.15);
    border-right: 1px solid rgba(255, 255, 255, 0.05);
    min-width: 0;
    box-sizing: border-box;
  }

  .cover-container {
    width: 100%;
    flex-shrink: 0;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
    background-color: #222;
    overflow: hidden;
  }

  .meta-container {
    margin-top: 28px;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .album-title {
    margin: 0;
    font-size: 26px;
    font-weight: 500;
    color: var(--text-main);
    line-height: 1.2;
    word-wrap: break-word;
  }

  .album-artist {
    margin: 6px 0 0 0;
    font-size: 18px;
    font-weight: 400;
    color: var(--text-muted);
    line-height: 1.2;
    word-wrap: break-word;
  }

  .actions-row {
    margin-top: 24px;
  }

  .play-all-btn {
    background: none;
    border: 1px solid var(--border-muted);
    color: var(--text-muted);
    padding: 8px 20px;
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.05em;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .play-all-btn:hover {
    color: var(--text-main);
    border-color: var(--text-main);
    background-color: rgba(255, 255, 255, 0.05);
  }

  .column-right {
    display: flex;
    flex-direction: column;
    padding: 32px;
    min-width: 0;
    height: 100%;
    box-sizing: border-box;
    background-color: var(--background-drawer);
  }

  .tracks-scroll-area {
    flex: 1;
    overflow-y: auto;
    padding-right: 12px;
  }
</style>
