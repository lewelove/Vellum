<script>
  import { library } from "../../library.svelte.js";
  import { GROUPER_LABELS } from "../../logic/groupers.js";
  import { SORTER_LABELS } from "../../logic/sorters.js";
  import SidebarItem from "./SidebarItem.svelte";

  let isSortMenuOpen = $state(false);
  let isGroupMenuOpen = $state(false);

  let groupLabel = $derived(GROUPER_LABELS[library.activeSidebarGrouper] || "Unknown");
  let sortLabel = $derived(SORTER_LABELS[library.userSortPreference] || "Unknown");

  let items = $derived(library.getSidebarGroup(library.activeSidebarGrouper));

  function handleMediaLibrary() {
    library.showMediaLibrary();
  }

  function handleRecentlyAdded() {
    library.showRecentlyAdded();
  }

  function toggleSortMenu() {
    isSortMenuOpen = !isSortMenuOpen;
    if (isSortMenuOpen) isGroupMenuOpen = false;
  }

  function toggleGroupMenu() {
    isGroupMenuOpen = !isGroupMenuOpen;
    if (isGroupMenuOpen) isSortMenuOpen = false;
  }

  function selectSorter(key) {
    library.setUserSort(key);
    isSortMenuOpen = false;
  }

  function selectGrouper(key) {
    library.setSidebarGrouper(key);
    isGroupMenuOpen = false;
  }
</script>

<div class="sidebar-container">
  <div class="sidebar-nav">
    <button class="nav-button" onclick={handleMediaLibrary}>
      Media Library
    </button>
    <button class="nav-button" onclick={handleRecentlyAdded}>
      Recently Added
    </button>
  </div>

  <div class="sidebar-controls">
    
    <div class="control-wrapper">
      <button class="control-toggle" onclick={toggleSortMenu} class:active={isSortMenuOpen}>
        <div class="control-label-group">
          <img src="/material/sort_24dp_FFFFFF.svg" alt="" class="control-icon" />
          <span class="control-label">{sortLabel}</span>
        </div>
        <span class="chevron" class:open={isSortMenuOpen}>›</span>
      </button>
  
      {#if isSortMenuOpen}
        <div class="control-menu">
          {#each Object.entries(SORTER_LABELS) as [key, label]}
            <button 
              class="menu-item" 
              class:selected={library.userSortPreference === key}
              onclick={() => selectSorter(key)}
            >
              {label}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="control-wrapper">
      <button class="control-toggle" onclick={toggleGroupMenu} class:active={isGroupMenuOpen}>
        <div class="control-label-group">
          <img src="/material/layers_24dp_FFFFFF.svg" alt="" class="control-icon" />
          <span class="control-label">{groupLabel}</span>
        </div>
        <span class="chevron" class:open={isGroupMenuOpen}>›</span>
      </button>
  
      {#if isGroupMenuOpen}
        <div class="control-menu">
          {#each Object.entries(GROUPER_LABELS) as [key, label]}
            <button 
              class="menu-item" 
              class:selected={library.activeSidebarGrouper === key}
              onclick={() => selectGrouper(key)}
            >
              {label}
            </button>
          {/each}
        </div>
      {/if}
    </div>

  </div>

  <div class="sidebar-scroll">
    {#each items as item}
      <SidebarItem 
        label={item.label} 
        count={item.count}
        active={library.activeFilter.key === item.filterTarget && library.activeFilter.val === item.value}
        onclick={() => library.applyFilter(item.filterTarget, item.value)}
      />
    {/each}
  </div>
</div>

<style>
.sidebar-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: #242424; 
    padding-top: 0; 
    box-sizing: border-box;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    padding: 12px 0 12px 0;
  }

  .nav-button {
    text-align: left;
    background: none;
    border: none;
    padding: 8px 16px 8px 24px;
    color: var(--text-main);
    font-family: var(--font-stack);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.1s;
    outline: none; 
  }

  .nav-button:hover {
    background-color: rgba(255, 255, 255, 0.05);
  }

  .sidebar-controls {
    display: flex;
    flex-direction: column;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .control-wrapper {
    position: relative;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .control-wrapper:last-child {
    border-bottom: none;
  }

  .control-toggle {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px 8px 20px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 14px;
    cursor: pointer;
    outline: none; 
  }

  .control-label-group {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .control-icon {
    width: 18px;
    height: 18px;
    opacity: 0.6;
    flex-shrink: 0;
  }

  .control-toggle:hover .control-icon,
  .control-toggle.active .control-icon {
    opacity: 1;
  }

  .control-label {
    text-transform: lowercase;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transform: translateY(-1px);
  }

  .control-toggle:hover {
    color: var(--text-main);
  }

  .control-toggle.active {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-main);
  }

  .chevron {
    transform: rotate(90deg);
    transition: transform 0.2s;
    font-size: 14px;
    flex-shrink: 0;
  }

  .chevron.open {
    transform: rotate(-90deg);
  }

  .control-menu {
    position: absolute;
    top: 100%;
    left: 0;
    width: 100%;
    background-color: #242424; 
    z-index: 20;
    box-shadow: 0 8px 8px rgba(0,0,0,0.3);
  }

  .menu-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 8px 24px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 14px;
    text-transform: lowercase;
    cursor: pointer;
    outline: none; 
  }

  .menu-item:hover {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-main);
  }

  .menu-item.selected {
    color: var(--text-main);
    font-weight: 400;
    background-color: rgba(255, 255, 255, 0.1);
  }

  .sidebar-scroll {
    flex: 1;
    overflow-y: auto;
    padding-top: 8px;
  }
</style>
