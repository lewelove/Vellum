<script>
  import { fade } from "svelte/transition";
  import { playAlbum } from "../../api.js";
  import { library } from "../../library.svelte.js";
  import SmartImage from "./SmartImage.svelte";
  import ModalDrawerTracks from "./ModalDrawerTracks.svelte";

  let { album, onclose } = $props();

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
      
      <div class="cover-column">
        <SmartImage src={coverUrl} width={400} height={400} />
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
    background-color: rgba(0, 0, 0, 0.1);
    backdrop-filter: blur(2px);
    -webkit-backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    box-sizing: border-box;
    will-change: opacity, backdrop-filter;
  }

  .modal-chassis {
    width: 80vw;
    height: 80vh;
    background-color: var(--background-drawer);
    box-shadow: 0 32px 64px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
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
    width: 40%;
    height: 40%;
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

  .play-all-btn {
    background: none;
    border: 1px solid var(--border-muted);
    color: var(--text-muted);
    padding: 6px 16px;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.1s;
  }

  .play-all-btn:hover {
    color: var(--text-main);
    background-color: rgba(255, 255, 255, 0.05);
    border-color: var(--text-main);
  }

  .album-artist {
    margin: 8px 0 0 0;
    font-size: 20px;
    font-weight: 400;
    color: var(--text-muted);
  }

  .tracks-scroll-area {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    padding-right: 12px;
  }

  .tracks-scroll-area::-webkit-scrollbar {
    width: 4px;
  }

  .tracks-scroll-area::-webkit-scrollbar-track {
    background: transparent;
  }

  .tracks-scroll-area::-webkit-scrollbar-thumb {
    background-color: var(--palette-300);
  }
</style>
