<script>
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

{#snippet NavButton({ icon, label, disabled, active, onclick })}
  <button class="v-btn-icon queue-nav-button" class:active {disabled} {onclick} title={label}>
    <img src="/{icon}" alt={label} class="nav-icon" />
  </button>
{/snippet}

<div class="queue-bar v-glass">
  <div class="nav-group top">
    {@render NavButton({ icon: "icons/outlined/24px/code.svg", label: "Open Data Object", disabled: !activeId, onclick: handleOpenLock })}
    {@render NavButton({ icon: "icons/outlined/24px/edit_document.svg", label: "Open Manifest", disabled: !activeId, onclick: handleOpenManifest })}
    {@render NavButton({ icon: "icons/outlined/24px/folder.svg", label: "Open Local Folder", disabled: !activeId, onclick: handleOpenFolder })}
    {@render NavButton({ icon: "icons/outlined/24px/refresh.svg", label: "Update Album", disabled: !activeId, onclick: handleUpdate })}
  </div>

  <div class="nav-group bottom">
    {@render NavButton({ icon: "icons/outlined/24px/menu_book.svg", label: "Lyrics", active: panels.lyrics, onclick: () => onToggle('lyrics') })}
    {@render NavButton({ icon: "icons/outlined/24px/format_list_bulleted.svg", label: "Track List", active: panels.tracks, onclick: () => onToggle('tracks') })}
    {@render NavButton({ icon: "icons/outlined/24px/colors.svg", label: "Toggle Shader", active: library.isShaderEnabled, onclick: () => library.toggleShader() })}
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

  .queue-nav-button {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    box-shadow: var(--button-shadow-lesser);
    flex-shrink: 0;
    pointer-events: auto;
  }

  .nav-icon {
    width: 24px;
    height: 24px;
  }
</style>
