<script>
  import { library } from "../../library.svelte.js";

  let isSortMenuOpen = $state(false);
  let isGroupMenuOpen = $state(false);

  let groupLabel = $derived(library.availableFacets[library.activeSidebarGrouper] || "Unknown");
  let sortLabel = $derived(library.availableSorters[library.userSortPreference] || "Unknown");

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

{#snippet Item({ label, count, active, onclick })}
  <button class="sidebar-item" class:active {onclick}>
    <span class="v-truncate label" title={label}>{label}</span>
    <span class="v-mono count">{count}</span>
  </button>
{/snippet}

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
          <img src="icons/20px/swap_vert.svg" alt="" class="btn-icon" />
          <span class="v-truncate btn-label">{sortLabel}</span>
          <img 
            src={isSortMenuOpen ? "icons/24px/arrow_drop_up.svg" : "icons/24px/arrow_drop_down.svg"} 
            class="chevron" 
            alt="" 
          />
        </button>

        {#if isSortMenuOpen}
          <div class="control-menu">
            {#each Object.entries(library.availableSorters) as [key, label]}
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
          class:mirrored={isReverse}
          src="/icons/24px/arrow_shape_up_stack_down.svg" 
          alt="Direction" 
        />
      </button>
    </div>

    <div class="control-row">
      <div class="button-wrapper flex-grow">
        <button class="sidebar-btn" onclick={toggleGroupMenu} class:active={isGroupMenuOpen} title="Group By">
          <img src="icons/20px/stack.svg" alt="" class="btn-icon" />
          <span class="v-truncate btn-label">{groupLabel}</span>
          <img 
            src={isGroupMenuOpen ? "icons/24px/arrow_drop_up.svg" : "icons/24px/arrow_drop_down.svg"}  
            class="chevron" 
            alt="" 
          />
        </button>
    
        {#if isGroupMenuOpen}
          <div class="control-menu">
            {#each Object.entries(library.availableFacets) as [key, label]}
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
    <div class="v-scroll-fade-top"></div>
    {#each items as item}
      {@render Item({
        label: item.label,
        count: item.count,
        active: library.activeFilter.key === item.filterTarget && library.activeFilter.val === item.value,
        onclick: () => library.applyFilter(item.filterTarget, item.value)
      })}
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

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    padding-bottom: 12px;
    gap: 4px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    margin-bottom: 12px;
    flex-shrink: 0;
  }

  .nav-button {
    text-align: right;
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
    gap: 8px;
    padding-bottom: 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
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

  .sidebar-btn {
    width: 100%;
    height: 36px;
    background-color: rgba(255, 255, 255, 0.01);
    border: 2px solid rgba(255, 255, 255, 0.08);
    padding: 0 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    cursor: pointer;
    border-radius: 10px;
    box-shadow: var(--button-shadow);
    transition: background-color 0.1s, border-color 0.1s;
    font-family: var(--font-stack);
    color: var(--text-muted);
    font-size: 14px;
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
    opacity: 0.7;
    margin-right: 8px;
    flex-shrink: 0;
    transition: opacity 0.1s;
  }
  
  .btn-icon.no-margin {
    margin-right: 0;
  }

  .btn-icon.mirrored {
    transform: scaleY(-1);
  }

  .sidebar-btn:hover .btn-icon, .sidebar-btn.active .btn-icon {
    opacity: 1;
  }

  .btn-label {
    flex: 1;
    text-align: left;
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
    margin-top: 6px;
    background-color: var(--background-drawer); 
    z-index: 50;
    box-shadow: 0 4px 12px rgba(0,0,0,0.4);
    border: 2px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 4px;
    box-sizing: border-box;
  }

  .menu-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 12px;
    margin-bottom: 2px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 14px;
    cursor: pointer;
    border-radius: 8px;
    outline: none;
    box-sizing: border-box;
    transition: background-color 0.1s ease;
  }

  .menu-item:hover {
    background-color: rgba(255, 255, 255, 0.03);
    color: var(--text-main);
  }

  .menu-item.selected {
    color: var(--text-main);
    background-color: rgba(255, 255, 255, 0.05);
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

  .count {
    opacity: 0.5;
    font-size: 13px;
  }
</style>
