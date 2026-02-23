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

  let isReverse = $derived(library.userSortOrder === "reverse");

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

  function toggleDirection() {
    library.toggleSortOrder();
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
    
    <div class="control-row">
      <div class="button-wrapper flex-grow">
        <button class="sidebar-btn" onclick={toggleSortMenu} class:active={isSortMenuOpen} title="Sort By">
          <img src="/material/swap_vert_20dp_FFFFFF.svg" alt="" class="btn-icon" />
          <span class="btn-label">{sortLabel}</span>
          <img 
            src={isSortMenuOpen ? "/material/arrow_drop_up_24dp_FFFFFF.svg" : "/material/arrow_drop_down_24dp_FFFFFF.svg"} 
            class="chevron" 
            alt="" 
          />
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
      
      <button class="sidebar-btn square" onclick={toggleDirection} title={isReverse ? "Reverse Order" : "Default Order"}>
        <img 
          class="btn-icon no-margin"
          src={isReverse ? "/material/arrow_upward_20dp_FFFFFF.svg" : "/material/arrow_downward_20dp_FFFFFF.svg"} 
          alt="Direction" 
        />
      </button>
    </div>

    <div class="control-row">
      <div class="button-wrapper flex-grow">
        <button class="sidebar-btn" onclick={toggleGroupMenu} class:active={isGroupMenuOpen} title="Group By">
          <img src="/material/stack_20dp_FFFFFF.svg" alt="" class="btn-icon" />
          <span class="btn-label">{groupLabel}</span>
          <img 
            src={isGroupMenuOpen ? "/material/arrow_drop_up_24dp_FFFFFF.svg" : "/material/arrow_drop_down_24dp_FFFFFF.svg"} 
            class="chevron" 
            alt="" 
          />
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
    padding: 12px 12px 0 12px; 
    box-sizing: border-box;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    padding-bottom: 12px;
    gap: 4px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    margin-bottom: 12px;
  }

  .nav-button {
    text-align: left;
    background: transparent;
    border: none;
    padding: 6px 12px;
    color: var(--text-main);
    font-family: var(--font-stack);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.1s;
    outline: none; 
    border-radius: 8px;
    box-sizing: border-box;
    width: 100%;
  }

  .nav-button:hover {
    background-color: rgba(255, 255, 255, 0.05);
  }

  .sidebar-controls {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-bottom: 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    margin-bottom: 4px;
  }

  .control-row {
    display: flex;
    gap: 10px;
    width: 100%;
  }

  .button-wrapper {
    position: relative;
  }

  .flex-grow {
    flex: 1;
    min-width: 0;
  }

  .sidebar-btn {
    width: 100%;
    height: 36px;
    background-color: rgba(255, 255, 255, 0.01);
    border: 1px solid rgba(255, 255, 255, 0.05);
    padding: 0 8px 0 10px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    cursor: pointer;
    border-radius: 8px;
    box-shadow: var(--button-shadow);
    transition: background-color 0.1s, border-color 0.1s;
    font-family: var(--font-stack);
    color: var(--text-muted);
    font-size: 13px;
    outline: none;
    box-sizing: border-box;
  }

  .sidebar-btn.square {
    width: 36px;
    padding: 0;
    justify-content: center;
    flex-shrink: 0;
  }

  .sidebar-btn:hover, .sidebar-btn.active {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-main);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .btn-icon {
    width: 20px;
    height: 20px;
    opacity: 0.7;
    margin-right: 8px;
    flex-shrink: 0;
    transition: opacity 0.1s;
  }
  
  .btn-icon.no-margin {
    margin-right: 0;
  }

  .sidebar-btn:hover .btn-icon, .sidebar-btn.active .btn-icon {
    opacity: 1;
  }

  .btn-label {
    flex: 1;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-transform: lowercase;
  }

  .chevron {
    width: 20px;
    height: 20px;
    margin-left: 4px;
    opacity: 0.5;
    flex-shrink: 0;
  }

  .sidebar-btn:hover .chevron, .sidebar-btn.active .chevron {
    opacity: 1;
  }

  .control-menu {
    position: absolute;
    top: 100%;
    left: 0;
    width: 100%;
    margin-top: 4px;
    background-color: #242424; 
    z-index: 50;
    box-shadow: 0 4px 12px rgba(0,0,0,0.4);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 4px;
    box-sizing: border-box;
  }

  .menu-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 8px 12px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 13px;
    text-transform: lowercase;
    cursor: pointer;
    border-radius: 4px;
    outline: none;
  }

  .menu-item:hover {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-main);
  }

  .menu-item.selected {
    color: var(--text-main);
    background-color: rgba(255, 255, 255, 0.1);
  }

  .sidebar-scroll {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 12px;
  }
</style>
