<script>
  import { fade } from "svelte/transition";
  import { playAlbum } from "../../api.js";
  import { library } from "../../library.svelte.js";
  import SmartImage from "./SmartImage.svelte";
  import ModalDrawerTracks from "./ModalDrawerTracks.svelte";

  let { album, onclose } = $props();

  // Programmatic Window Tracking
  let innerHeight = $state(window.innerHeight);

  let coverUrl = $derived(library.getAlbumCoverUrl(album.id));

  /**
   * PROGRAMMATIC DIMENSION CALCULATION
   * Modal height is 80vh (0.8 * innerHeight)
   * Content padding is 32px top + 32px bottom (64px total)
   */
  const PADDING = 64; 
  let sideLength = $derived(Math.floor((innerHeight * 0.7) - PADDING));

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

<svelte:window bind:innerHeight />

<div 
  class="modal-backdrop" 
  onclick={handleBackdropClick} 
  role="presentation"
  transition:fade={{ duration: 100 }}
>
  <div class="modal-chassis">
    <div class="modal-content">
      
      <div class="cover-column" style="width: {sideLength}px;">
        <SmartImage 
          src={coverUrl} 
          width={sideLength} 
          height={sideLength} 
        />
      </div>

      <div class="info-column">
        <div class="header-section">
          <div class="title-line">
            <h2 class="album-title">{album.title}</h2>
            <button class="play-all-btn" onclick={handlePlay}>PLAY ALBUM</button>
          </div>
          <h3 class="album-artist">{album.artist}</h3>
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
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.05);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-chassis {
    width: 75vw;
    height: 70vh;
    background-color: var(--background-drawer);
    box-shadow: 0 0px 64px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    radius: 8px;
  }

  .modal-content {
    display: flex;
    flex-direction: row;
    height: 100%;
    padding: 32px;
    gap: 32px;
    box-sizing: border-box;
  }

  .cover-column {
    flex-shrink: 0;
    height: 100%;
    display: flex;
  }

  .info-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
  }

  .header-section {
    flex-shrink: 0;
    margin-bottom: 24px;
  }

  .title-line {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
  }

  .album-title {
    margin: 0;
    font-size: 28px;
    font-weight: 400;
    color: var(--text-main);
  }

  .album-artist {
    margin: 8px 0 0 0;
    font-size: 20px;
    font-weight: 400;
    color: var(--text-muted);
  }

  .play-all-btn {
    background: none;
    border: 1px solid var(--border-muted);
    color: var(--text-muted);
    padding: 6px 16px;
    font-size: 12px;
    cursor: pointer;
  }

  .tracks-scroll-area {
    flex: 1;
    overflow-y: auto;
    padding-right: 12px;
  }

  /* Webkit scrollbar styles omitted for brevity, keep your existing ones */
</style>
