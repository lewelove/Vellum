<script>
  import { library } from "$state/library.svelte.js";
  import { GROUPER_LABELS } from "../../logic/groupers.js";
  import { SORTER_LABELS } from "../../logic/sorters.js";
  import SidebarItem from "./SidebarItem.svelte";

  let isSortMenuOpen = $state(false);
  let isGroupMenuOpen = $state(false);

  let groupLabel = $derived(GROUPER_LABELS[library.activeSidebarGrouper] || "Unknown");
  // The label displayed depends on the *user's preference*, not necessarily the active override 
  // (though usually they match in Media Library view).
  let sortLabel = $derived(SORTER_LABELS[library.userSortPreference] || "Unknown");

  // Derive items based on active grouper
  let items = $derived(library.getSidebarGroup(library.activeSidebarGrouper));

  function handleMediaLibrary() {
    library.applyFilter(null, null);
    library.restoreUserSort();
  }

  function handleRecentlyAdded() {
    library.applyFilter(null, null);
    library.applySort("date_added");
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
  <!-- 1. Static Navigation -->
  <div class="sidebar-nav">
    <button class="nav-button" onclick={handleMediaLibrary}>
      Media Library
    </button>
    <button class="nav-button" onclick={handleRecentlyAdded}>
      Recently Added
    </button>
  </div>

  <!-- 2. Controls Section -->
  <div class="sidebar-controls">
    
    <!-- Sort Toggle -->
    <div class="control-wrapper">
      <button class="control-toggle" onclick={toggleSortMenu} class:active={isSortMenuOpen}>
        <span class="control-label">Sort: {sortLabel}</span>
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

    <!-- Group Toggle -->
    <div class="control-wrapper">
      <button class="control-toggle" onclick={toggleGroupMenu} class:active={isGroupMenuOpen}>
        <span class="control-label">Group: {groupLabel}</span>
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

  <!-- 3. Dynamic List -->
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
    background-color: var(--background-drawer);
  }

  /* Navigation Section */
  .sidebar-nav {
    display: flex;
    flex-direction: column;
    padding: 24px 0 12px 0;
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

  /* Controls Section */
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
    padding: 12px 16px 12px 24px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
    outline: none; 
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
  }

  .chevron.open {
    transform: rotate(-90deg);
  }

  /* Dropdown Menu */
  .control-menu {
    position: absolute;
    top: 100%;
    left: 0;
    width: 100%;
    background-color: #1a1a1a; 
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    z-index: 20;
    box-shadow: 0 4px 6px rgba(0,0,0,0.3);
  }

  .menu-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 10px 24px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 13px;
    cursor: pointer;
    outline: none; 
  }

  .menu-item:hover {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-main);
  }

  .menu-item.selected {
    color: var(--text-main);
    font-weight: 500;
    background-color: rgba(255, 255, 255, 0.1);
  }

  /* Scroll List */
  .sidebar-scroll {
    flex: 1;
    overflow-y: auto;
    padding-top: 8px;
  }
</style>
