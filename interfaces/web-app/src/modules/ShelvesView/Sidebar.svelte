<script lang="ts">
  import { view } from "../../library/view.svelte.ts";
  import { collection } from "../../library/collection.svelte.ts";

  let isShelfMenuOpen = $state(false);

  let activeShelf = $derived(view.activeShelf || (collection.manifest.shelves_order && collection.manifest.shelves_order[0]) || Object.keys(collection.availableShelves)[0]);
  let shelfLabel = $derived(collection.availableShelves[activeShelf]?.label || "Unknown");

  function selectShelf(key: string) {
    view.setShelf(key);
    isShelfMenuOpen = false;
  }

  function toggleSubView() {
    view.homeSubView = view.homeSubView === "library" ? "shelves" : "library";
    view.focusedAlbum = null;
    view.refreshView(true);
    view.persistState();
  }

  function toggleShelfMenu() {
    isShelfMenuOpen = !isShelfMenuOpen;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      isShelfMenuOpen = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="sidebar-container">
  <div class="sidebar-controls">
    <div class="control-row">
      <button class="v-btn-icon v-sidebar-button" onclick={toggleSubView} title={view.homeSubView === 'library' ? "Libraries" : "Shelves"}>
        <span class="icon">{view.homeSubView === 'library' ? "auto_stories" : "newsstand"}</span>
      </button>

      <div class="button-wrapper flex-grow">
        <button class="v-btn-icon v-sidebar-button v-btn-menu" onclick={toggleShelfMenu} class:active={isShelfMenuOpen} title="Shelf">
          <span class="v-truncate btn-label iconless">{shelfLabel}</span>
          <span class="icon end-icon">{isShelfMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
        </button>

        {#if isShelfMenuOpen}
          <div class="v-menu">
            {#each collection.shelvesList as shelf}
              <button 
                class="v-menu-item" 
                class:selected={activeShelf === shelf.key}
                onclick={() => selectShelf(shelf.key)}
              >
                {shelf.label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <div class="sidebar-scroll">
    <div class="v-scroll-fade-top"></div>
    {#each collection.shelvesList as shelf}
      <button 
        class="v-sidebar-item" 
        class:active={activeShelf === shelf.key} 
        onclick={() => selectShelf(shelf.key)}
      >
        <span class="v-truncate label" title={shelf.label}>{shelf.label}</span>
      </button>
    {/each}
    <div class="scroll-spacer"></div>
    <div class="v-scroll-fade-bottom"></div>
  </div>
</div>

<style>
  .sidebar-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-panel); 
    padding: 10px; 
    box-sizing: border-box;
    font-family: var(--font-stack);
  }

  .sidebar-controls {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-muted);
    margin-bottom: 0px;
    flex-shrink: 0;
  }

  .control-row {
    display: flex;
    gap: 8px;
    width: 100%;
  }

  .button-wrapper {
    position: relative;
  }

  .flex-grow {
    flex: 1;
    min-width: 0;
  }

  .sidebar-scroll {
    position: relative;
    flex: 1;
    overflow-y: scroll;
    padding: 0;
    min-height: 0;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .sidebar-scroll::-webkit-scrollbar {
    display: none;
  }

  .scroll-spacer {
    height: 12px;
    flex-shrink: 0;
  }
</style>
