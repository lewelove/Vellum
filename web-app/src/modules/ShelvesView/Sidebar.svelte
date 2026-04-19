<script>
  import { library } from "../../library.svelte.js";

  let activeShelf = $derived(library.activeShelf || Object.keys(library.availableShelves)[0]);

  function selectShelf(key) {
    library.setShelf(key);
  }
</script>

<div class="sidebar-container">
  <div class="sidebar-controls">
    <div class="sidebar-header">
      <img src="icons/outlined/20px/auto_stories.svg" alt="" class="header-icon" />
      <span class="header-title">Shelves</span>
    </div>
  </div>

  <div class="sidebar-scroll">
    <div class="v-scroll-fade-top"></div>
    {#each Object.entries(library.availableShelves) as [key, shelf]}
      <button 
        class="sidebar-item" 
        class:active={activeShelf === key} 
        onclick={() => selectShelf(key)}
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
    background-color: var(--background-drawer); 
    padding: 12px; 
    box-sizing: border-box;
  }

  .sidebar-controls {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-bottom: 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    margin-bottom: 0px;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    padding: 0 8px;
    height: 36px;
    color: var(--text-muted);
  }

  .header-icon {
    opacity: 0.7;
    margin-right: 8px;
    flex-shrink: 0;
  }

  .header-title {
    font-size: 14px;
    font-weight: 500;
  }

  .sidebar-scroll {
    position: relative;
    flex: 1;
    overflow-y: scroll;
    padding: 0;
    min-height: 0;
    scrollbar-width: none;
    -ms-overflow-style: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .sidebar-scroll::-webkit-scrollbar {
    display: none;
  }

  .scroll-spacer {
    height: 12px;
    flex-shrink: 0;
  }

  .sidebar-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    background-color: transparent;
    border: none;
    padding: 6px 12px;
    margin-bottom: 2px;
    cursor: default;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 14px;
    text-align: left;
    transition: background-color 0.1s ease;
    outline: none;
    border-radius: 8px;
    box-sizing: border-box;
    user-select: none;
  }

  .sidebar-item:hover {
    background-color: rgba(255, 255, 255, 0.03);
    color: var(--text-main);
    cursor: pointer;
  }

  .sidebar-item.active {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-main);
  }

  .label {
    flex: 1;
    margin-right: 8px;
  }
</style>
