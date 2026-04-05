<script>
  import QueueNavButton from "./QueueNavButton.svelte";
  import { library } from "../../library.svelte.js";
  import { player } from "../player.svelte.js";
  import { 
    playAlbum, 
    openAlbumFolder, 
    openLockFile, 
    openManifestFile, 
    updateAlbum 
  } from "../../api.js";

  let { panels, onToggle } = $props();

  let activeId = $derived(player.currentAlbumId);

  async function handlePlay() {
    if (activeId) await playAlbum(activeId);
  }

  async function handleOpenFolder() {
    if (activeId) await openAlbumFolder(activeId);
  }

  async function handleOpenLock() {
    if (activeId) await openLockFile(activeId);
  }

  async function handleOpenManifest() {
    if (activeId) await openManifestFile(activeId);
  }

  async function handleUpdate() {
    if (activeId) await updateAlbum(activeId);
  }
</script>

<div class="queue-bar v-glass">
  <div class="nav-group top">
    <QueueNavButton 
      icon="icons/24px/code.svg" 
      label="Open Data Object" 
      disabled={!activeId}
      onclick={handleOpenLock} 
    />
    <QueueNavButton 
      icon="icons/24px/edit_document.svg" 
      label="Open Manifest" 
      disabled={!activeId}
      onclick={handleOpenManifest} 
    />
    <QueueNavButton 
      icon="icons/24px/folder.svg" 
      label="Open Local Folder" 
      disabled={!activeId}
      onclick={handleOpenFolder} 
    />
    <QueueNavButton 
      icon="icons/24px/refresh.svg" 
      label="Update Album" 
      disabled={!activeId}
      onclick={handleUpdate} 
    />
  </div>

  <div class="nav-group bottom">
    <QueueNavButton 
      icon="icons/outlined/20px/menu_book.svg" 
      label="Lyrics" 
      active={panels.lyrics}
      onclick={() => onToggle('lyrics')} 
    />
    <QueueNavButton 
      icon="icons/outlined/24px/format_list_bulleted.svg" 
      label="Track List" 
      active={panels.tracks}
      onclick={() => onToggle('tracks')} 
    />
    <QueueNavButton 
      icon="icons/24px/colors.svg" 
      label="Toggle Shader" 
      active={library.isShaderEnabled}
      onclick={() => library.toggleShader()} 
    />
  </div>
</div>

<style>
  .queue-bar {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    box-sizing: border-box;
    z-index: 100;
    flex-shrink: 0;
  }
  
  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
