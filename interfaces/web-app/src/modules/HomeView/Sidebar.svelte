<script lang="ts">
  import { view } from "../../library/view.svelte.ts";
  import { collection } from "../../library/collection.svelte.ts";
  import SidebarIndex from "./SidebarIndex.svelte";

  let isLibraryMenuOpen = $state(false);
  let isLibraryFilterMenuOpen = $state(false);
  let isSortMenuOpen = $state(false);
  let isGroupMenuOpen = $state(false);
  let scrollContainer: HTMLDivElement | null = $state(null);

  let activeLibraryDef = $derived(collection.availableLibraries[view.activeLibrary] || {});
  let allowedFilters = $derived(activeLibraryDef.allowed_filters || []);
  let showFilterDropdown = $derived(allowedFilters.length > 1);

  let libraryLabel = $derived(collection.availableLibraries[view.activeLibrary]?.label || "Unknown");
  let filterLabel = $derived(collection.availableFilters[view.activeLibraryFilter || ""]?.label || "Unknown");
  let groupLabel = $derived(collection.availableFacets[view.activeSidebarGrouper]?.label || "Unknown");
  let sortLabel = $derived(collection.availableOrders[view.userSortPreference]?.label || "Unknown");

  let items = $derived(view.getSidebarGroup(view.activeSidebarGrouper));

  let isReverse = $derived(view.userSortOrder === "reverse");

  let activeGrouperDef = $derived(collection.availableFacets[view.activeSidebarGrouper] || {});
  let showIndex = $derived(activeGrouperDef.index === true);
  let showCount = $derived(activeGrouperDef.count === true);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      isLibraryMenuOpen = false;
      isLibraryFilterMenuOpen = false;
      isSortMenuOpen = false;
      isGroupMenuOpen = false;
    }
  }

  function toggleSubView() {
    view.homeSubView = view.homeSubView === "library" ? "shelves" : "library";
    view.focusedAlbum = null;
    view.refreshView(true);
    view.persistState();
  }

  function toggleLibraryMenu() {
    isLibraryMenuOpen = !isLibraryMenuOpen;
    if (isLibraryMenuOpen) {
      isLibraryFilterMenuOpen = false;
      isSortMenuOpen = false;
      isGroupMenuOpen = false;
    }
  }

  function toggleLibraryFilterMenu() {
    isLibraryFilterMenuOpen = !isLibraryFilterMenuOpen;
    if (isLibraryFilterMenuOpen) {
      isLibraryMenuOpen = false;
      isSortMenuOpen = false;
      isGroupMenuOpen = false;
    }
  }

  function toggleSortMenu() {
    isSortMenuOpen = !isSortMenuOpen;
    if (isSortMenuOpen) {
      isLibraryMenuOpen = false;
      isLibraryFilterMenuOpen = false;
      isGroupMenuOpen = false;
    }
  }

  function toggleGroupMenu() {
    isGroupMenuOpen = !isGroupMenuOpen;
    if (isGroupMenuOpen) {
      isLibraryMenuOpen = false;
      isLibraryFilterMenuOpen = false;
      isSortMenuOpen = false;
    }
  }

  function selectLibrary(key: string) {
    view.setLibrary(key);
    isLibraryMenuOpen = false;
  }

  function selectOrder(key: string) {
    view.setUserSort(key);
    isSortMenuOpen = false;
  }

  function selectGrouper(key: string) {
    view.setSidebarGrouper(key);
    isGroupMenuOpen = false;
  }

  function toggleDirection() {
    view.toggleSortOrder();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#snippet Item({ index, label, count, active, onclick }: { index: number, label: string, count: number, active: boolean, onclick: () => void })}
  <button id="sidebar-item-{index}" class="sidebar-item" class:active {onclick}>
    <span class="v-truncate label" title={label}>{label}</span>
    {#if showCount}
      <span class="count">{count}</span>
    {/if}
  </button>
{/snippet}

<div class="sidebar-container">

  <div class="sidebar-controls">
    <div class="control-row">
      <button class="v-btn-icon v-sidebar-button" onclick={toggleSubView} title={view.homeSubView === 'library' ? "Libraries" : "Shelves"}>
        <span class="icon">{view.homeSubView === 'library' ? "auto_stories" : "newsstand"}</span>
      </button>

      <div class="button-wrapper flex-grow">
        <button class="v-btn-icon v-sidebar-button sidebar-btn" onclick={toggleLibraryMenu} class:active={isLibraryMenuOpen} title="Library">
          <span class="v-truncate btn-label iconless">{libraryLabel}</span>
          <span class="icon end-icon">{isLibraryMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
        </button>
    
        {#if isLibraryMenuOpen}
          <div class="v-menu">
            {#each collection.librariesList as lib}
              <button 
                class="v-menu-item" 
                class:selected={view.activeLibrary === lib.key}
                onclick={() => selectLibrary(lib.key)}
              >
                {lib.label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    {#if showFilterDropdown}
      <div class="control-row">
        <div class="button-wrapper flex-grow">
          <button class="v-btn-icon v-sidebar-button sidebar-btn" onclick={toggleLibraryFilterMenu} class:active={isLibraryFilterMenuOpen} title="Filter">
            <span class="icon start-icon">texture</span>
            <span class="v-truncate btn-label">{filterLabel}</span>
            <span class="icon end-icon">{isLibraryFilterMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
          </button>
      
          {#if isLibraryFilterMenuOpen}
            <div class="v-menu">
              {#each allowedFilters as fKey}
                <button 
                  class="v-menu-item" 
                  class:selected={view.activeLibraryFilter === fKey}
                  onclick={() => {
                    view.setLibraryFilter(fKey);
                    isLibraryFilterMenuOpen = false;
                  }}
                >
                  {collection.availableFilters[fKey]?.label || fKey}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <div class="control-row">
      <div class="button-wrapper flex-grow">
        <button class="v-btn-icon v-sidebar-button sidebar-btn" onclick={toggleGroupMenu} class:active={isGroupMenuOpen} title="Group By">
          <span class="icon start-icon">stack_group</span>
          <span class="v-truncate btn-label">{groupLabel}</span>
          <span class="icon end-icon">{isGroupMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
        </button>
    
        {#if isGroupMenuOpen}
          <div class="v-menu">
            {#each collection.getVisibleFacets(view.activeLibrary) as {key, label}}
              <button 
                class="v-menu-item" 
                class:selected={view.activeSidebarGrouper === key}
                onclick={() => selectGrouper(key)}
              >
                {label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <div class="control-row">
      <div class="button-wrapper flex-grow">
        <button class="v-btn-icon v-sidebar-button sidebar-btn" onclick={toggleSortMenu} class:active={isSortMenuOpen} title="Sort By">
          <span class="icon start-icon">swap_vert</span>
          <span class="v-truncate btn-label">{sortLabel}</span>
          <span class="icon end-icon">{isSortMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
        </button>

        {#if isSortMenuOpen}
          <div class="v-menu">
            {#each collection.getVisibleOrders(view.activeLibrary) as {key, label}}
              <button 
                class="v-menu-item" 
                class:selected={view.userSortPreference === key}
                onclick={() => selectOrder(key)}
              >
                {label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
      
      <button class="v-btn-icon v-sidebar-button" onclick={toggleDirection} title={isReverse ? "Reverse Order" : "Default Order"}>
        <span class="icon order-arrow" class:mirrored={isReverse}>arrow_shape_up_stack</span>
      </button>
    </div>
  </div>

  <div class="sidebar-body">
    <div class="sidebar-scroll" bind:this={scrollContainer}>
      <div class="v-scroll-fade-top"></div>
      {#each items as item, i}
        {@render Item({
          index: i,
          label: item.label,
          count: item.count,
          active: view.activeFilter.key === view.activeSidebarGrouper && view.activeFilter.val === item.value,
          onclick: () => view.applyFilter(view.activeSidebarGrouper, item.value)
        })}
      {/each}
      <div class="scroll-spacer"></div>
      <div class="v-scroll-fade-bottom"></div>
    </div>
    
    {#if items.length > 0 && showIndex}
      <SidebarIndex {items} container={scrollContainer} />
    {/if}
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

  .sidebar-btn {
    width: 100%;
    padding: 0 8px;
    justify-content: space-between;
    color: var(--text-muted);
    font-size: 16px;
    font-family: var(--font-stack);
    font-weight: 500;
  }

  .order-arrow.mirrored {
    transform: scaleY(-1);
  }

  .start-icon {
    font-size: 20px;
    margin-right: 6px;
    flex-shrink: 0;
  }

  .end-icon {
    font-size: 20px;
    margin-left: 4px;
    flex-shrink: 0;
  }

  .btn-label {
    flex: 1;
    text-align: left;
    font-family: var(--font-stack);
  }

  .btn-label.iconless {
    padding-left: 2px;
  }

  .sidebar-body {
    position: relative;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: row;
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

  .sidebar-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    background-color: transparent;
    border: none;
    padding: 6px 10px;
    margin-bottom: 4px;
    cursor: pointer;
    color: var(--text-muted);
    font-size: 14px;
    font-family: var(--font-stack);
    text-align: left;
    border-radius: 8px;
    box-sizing: border-box;
  }

  .sidebar-item:hover {
    background-color: var(--bg-item-hover);
    color: var(--text-main);
  }

  .sidebar-item.active {
    background-color: var(--bg-item-active);
    color: var(--text-main);
  }

  .label {
    flex: 1;
    margin-right: 8px;
    font-family: var(--font-stack);
  }

  .count {
    margin-left: 8px;
    font-size: 12px;
    color: var(--text-subtle);
    font-family: var(--font-mono);
  }
</style>
