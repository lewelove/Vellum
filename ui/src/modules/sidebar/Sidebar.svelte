<script>
  import { library } from "$state/library.svelte.js";
  import { GROUPER_LABELS } from "../../logic/groupers.js";
  import SidebarItem from "./SidebarItem.svelte";

  let isGroupMenuOpen = $state(false);
  let groupLabel = $derived(GROUPER_LABELS[library.activeSidebarGrouper] || "Unknown");

  // Derive items based on active grouper
  let items = $derived(library.getSidebarGroup(library.activeSidebarGrouper));

  function handleMediaLibrary() {
    library.applyFilter(null, null);
    library.applySort("default");
  }

  function handleRecentlyAdded() {
    library.applyFilter(null, null);
    library.applySort("date_added");
  }

  function toggleGroupMenu() {
    isGroupMenuOpen = !isGroupMenuOpen;
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

  <!-- 2. Grouping Controls -->
  <div class="sidebar-controls">
    <button class="group-toggle" onclick={toggleGroupMenu} class:active={isGroupMenuOpen}>
      <span class="group-label">Group: {groupLabel}</span>
      <span class="chevron" class:open={isGroupMenuOpen}>›</span>
    </button>

    {#if isGroupMenuOpen}
      <div class="group-menu">
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
    outline: none; /* Removes focus border */
  }

  .nav-button:hover {
    background-color: rgba(255, 255, 255, 0.05);
  }

  /* Controls Section */
  .sidebar-controls {
    position: relative;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .group-toggle {
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
    outline: none; /* Removes focus border */
  }

  .group-toggle:hover {
    color: var(--text-main);
  }

  .group-toggle.active {
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
  .group-menu {
    position: absolute;
    top: 100%;
    left: 0;
    width: 100%;
    background-color: #1a1a1a; /* Slightly lighter than pure background */
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
    outline: none; /* Removes focus border */
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
