<script lang="ts">
import { SvelteSet } from "svelte/reactivity";
import { view } from "../../library/view.svelte.ts";
import { collection } from "../../library/collection.svelte.ts";
import SidebarIndex from "./SidebarIndex.svelte";

let isLibraryMenuOpen = $state(false);
let isLibraryFilterMenuOpen = $state(false);
let isSortMenuOpen = $state(false);
let isGroupMenuOpen = $state(false);
let isCabinetMenuOpen = $state(false);
let isCabinetOrderMenuOpen = $state(false);
let scrollContainer: HTMLDivElement | null = $state(null);
let expandedNodes = new SvelteSet<string>();

let isCabinetsMode = $derived(view.homeSubView === "cabinets");

let visibleFilters = $derived(collection.getVisibleFilters(view.activeLibrary));
let showFilterDropdown = $derived(visibleFilters.length > 0);

let visibleGroupers = $derived(collection.getVisibleGroupers(view.activeLibrary));
let showGroupDropdown = $derived(visibleGroupers.length > 0);

let visibleOrders = $derived(collection.getVisibleOrders(view.activeLibrary));
let showSortDropdown = $derived(visibleOrders.length > 0);

let libraryLabel = $derived(collection.availableLibraries[view.activeLibrary]?.label || "Unknown");
let filterLabel = $derived(
  collection.availableFilters[view.activeLibraryFilter || ""]?.label || "Unknown"
);
let groupLabel = $derived(
  view.activeSidebarGrouper
    ? collection.availableGroupers[view.activeSidebarGrouper]?.label || "Unknown"
    : ""
);
let sortLabel = $derived(
  view.userSortPreference
    ? collection.availableOrders[view.userSortPreference]?.label || "Unknown"
    : ""
);

let activeCabinet = $derived(
  view.activeCabinet ||
    (collection.manifest.cabinets_order && collection.manifest.cabinets_order[0]) ||
    Object.keys(collection.availableCabinets)[0] ||
    "default"
);
let cabinetLabel = $derived(collection.availableCabinets[activeCabinet]?.label || "Cabinet");

let visibleShelves = $derived(collection.getVisibleShelvesForCabinet(activeCabinet));
let activeShelf = $derived(
  view.activeShelf || (visibleShelves.length > 0 ? visibleShelves[0].key : "")
);

let visibleCabinetOrders = $derived(collection.getVisibleOrdersForCabinet(activeCabinet));
let showCabinetOrderDropdown = $derived(visibleCabinetOrders.length >= 1);
let cabinetOrderLabel = $derived(
  view.activeShelfOrder === "original"
    ? "Original"
    : collection.availableOrders[view.activeShelfOrder]?.label || "Original"
);

let items = $derived(view.getSidebarGroup(view.activeSidebarGrouper));

let isReverse = $derived(view.userSortOrder === "reverse");

let activeGrouperDef = $derived(
  view.activeSidebarGrouper ? collection.availableGroupers[view.activeSidebarGrouper] || {} : {}
);
let showIndex = $derived(activeGrouperDef.index === true);
let showCount = $derived(activeGrouperDef.count === true);

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    closeAllMenus();
  }
}

function closeAllMenus() {
  isLibraryMenuOpen = false;
  isLibraryFilterMenuOpen = false;
  isSortMenuOpen = false;
  isGroupMenuOpen = false;
  isCabinetMenuOpen = false;
  isCabinetOrderMenuOpen = false;
}

function toggleSubView() {
  closeAllMenus();
  view.homeSubView = view.homeSubView === "libraries" ? "cabinets" : "libraries";
  view.focusedAlbum = null;
  view.refreshView(true);
  view.persistState();
}

function toggleLibraryMenu() {
  const target = !isLibraryMenuOpen;
  closeAllMenus();
  isLibraryMenuOpen = target;
}

function toggleLibraryFilterMenu() {
  const target = !isLibraryFilterMenuOpen;
  closeAllMenus();
  isLibraryFilterMenuOpen = target;
}

function toggleSortMenu() {
  const target = !isSortMenuOpen;
  closeAllMenus();
  isSortMenuOpen = target;
}

function toggleGroupMenu() {
  const target = !isGroupMenuOpen;
  closeAllMenus();
  isGroupMenuOpen = target;
}

function toggleCabinetMenu() {
  const target = !isCabinetMenuOpen;
  closeAllMenus();
  isCabinetMenuOpen = target;
}

function toggleCabinetOrderMenu() {
  const target = !isCabinetOrderMenuOpen;
  closeAllMenus();
  isCabinetOrderMenuOpen = target;
}

function selectLibrary(key: string) {
  view.setLibrary(key);
  isLibraryMenuOpen = false;
}

function selectCabinet(key: string) {
  view.setCabinet(key);
  isCabinetMenuOpen = false;
}

function selectOrder(key: string) {
  view.setUserSort(key);
  isSortMenuOpen = false;
}

function selectCabinetOrder(key: string) {
  view.setShelfOrder(key);
  isCabinetOrderMenuOpen = false;
}

function selectGrouper(key: string) {
  view.setSidebarGrouper(key);
  isGroupMenuOpen = false;
}

function toggleDirection() {
  view.toggleSortOrder();
}

function toggleCabinetDirection() {
  view.toggleShelfOrderDirection();
}

function toggleExpand(value: string) {
  if (expandedNodes.has(value)) {
    expandedNodes.delete(value);
  } else {
    expandedNodes.add(value);
  }
}

function hasSelectedChild(node: any, selectedVal: string | null): boolean {
  if (!selectedVal || !node.children || node.children.length === 0) return false;
  for (const child of node.children) {
    if (child.value === selectedVal || hasSelectedChild(child, selectedVal)) {
      return true;
    }
  }
  return false;
}
</script>

<svelte:window onkeydown={handleKeydown} />

{#snippet TreeNode(item: any, idPrefix: string, depth: number)}
  {@const hasChildren = Boolean(item.children && item.children.length > 0)}
  {@const isExpanded = expandedNodes.has(item.value)}
  {@const activeGrouper = view.activeSidebarGrouper}
  {@const currentFilterVal =
    Boolean(activeGrouper) && view.activeFilter.key === activeGrouper
      ? view.activeFilter.val
      : null}
  {@const isSelfActive = currentFilterVal !== null && currentFilterVal === item.value}
  {@const isChildActive = currentFilterVal !== null && hasSelectedChild(item, currentFilterVal)}
  {@const isActive = isSelfActive || isChildActive}

  <div class="v-tree-node">
    <div
      class="v-tree-row"
      class:active={isActive}
      class:expanded={isExpanded}
      style:padding-left="{depth * 8}px"
    >
      <button
        type="button"
        id="sidebar-item-{idPrefix}"
        class="v-sidebar-item"
        class:active={isActive}
        onclick={() => {
          if (view.activeSidebarGrouper) {
            view.applyFilter(view.activeSidebarGrouper, item.value);
          }
        }}
      >
        <span class="v-truncate label" title={item.label}>{item.label}</span>
        {#if showCount}
          <span class="count">{item.count}</span>
        {/if}
      </button>

      {#if hasChildren}
        <button
          type="button"
          class="v-tree-toggle"
          class:active={isActive}
          class:expanded={isExpanded}
          onclick={() => toggleExpand(item.value)}
          title={isExpanded ? "Collapse" : "Expand"}
        >
          <span class="icon tree-icon">{isExpanded ? "arrow_drop_up" : "arrow_drop_down"}</span>
        </button>
      {/if}
    </div>

    {#if hasChildren && isExpanded}
      <div class="v-tree-children">
        {#each item.children as child, childIdx (child.value || child.label || childIdx)}
          {@render TreeNode(child, `${idPrefix}-${childIdx}`, depth + 1)}
        {/each}
      </div>
    {/if}
  </div>
{/snippet}

<div class="v-sidebar-container">
  <div class="v-sidebar-controls">
    <div class="v-control-row">
      <button
        type="button"
        class="v-btn-icon v-sidebar-button"
        onclick={toggleSubView}
        title={isCabinetsMode ? "Cabinets" : "Libraries"}
      >
        <span class="icon">{isCabinetsMode ? "shelves" : "auto_stories"}</span>
      </button>

      {#if isCabinetsMode}
        {#if collection.cabinetsList.length > 0}
          <div class="v-button-wrapper v-flex-grow">
            <button
              type="button"
              class="v-btn-icon v-sidebar-button v-btn-menu"
              onclick={toggleCabinetMenu}
              class:active={isCabinetMenuOpen}
              title="Cabinet"
            >
              <span class="v-truncate btn-label iconless">{cabinetLabel}</span>
              <span class="icon end-icon">{isCabinetMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
            </button>

            {#if isCabinetMenuOpen}
              <div class="v-menu">
                {#each collection.cabinetsList as cab (cab.key)}
                  <button
                    type="button"
                    class="v-menu-item"
                    class:selected={activeCabinet === cab.key}
                    onclick={() => selectCabinet(cab.key)}
                  >
                    {cab.label}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      {:else}
        <div class="v-button-wrapper v-flex-grow">
          <button
            type="button"
            class="v-btn-icon v-sidebar-button v-btn-menu"
            onclick={toggleLibraryMenu}
            class:active={isLibraryMenuOpen}
            title="Library"
          >
            <span class="v-truncate btn-label iconless">{libraryLabel}</span>
            <span class="icon end-icon">{isLibraryMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
          </button>

          {#if isLibraryMenuOpen}
            <div class="v-menu">
              {#each collection.librariesList as lib (lib.key)}
                <button
                  type="button"
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
      {/if}
    </div>

    {#if isCabinetsMode}
      {#if showCabinetOrderDropdown}
        <div class="v-control-row">
          <div class="v-button-wrapper v-flex-grow">
            <button
              type="button"
              class="v-btn-icon v-sidebar-button v-btn-menu"
              onclick={toggleCabinetOrderMenu}
              class:active={isCabinetOrderMenuOpen}
              title="Sort By"
            >
              <span class="icon start-icon">swap_vert</span>
              <span class="v-truncate btn-label">{cabinetOrderLabel}</span>
              <span class="icon end-icon">{isCabinetOrderMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
            </button>

            {#if isCabinetOrderMenuOpen}
              <div class="v-menu">
                <button
                  type="button"
                  class="v-menu-item"
                  class:selected={view.activeShelfOrder === "original"}
                  onclick={() => selectCabinetOrder("original")}
                >
                  Original
                </button>
                {#each visibleCabinetOrders as { key, label } (key)}
                  <button
                    type="button"
                    class="v-menu-item"
                    class:selected={view.activeShelfOrder === key}
                    onclick={() => selectCabinetOrder(key)}
                  >
                    {label}
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <button
            type="button"
            class="v-btn-icon v-sidebar-button"
            onclick={toggleCabinetDirection}
            title={view.activeShelfOrderReverse ? "Reverse Order" : "Default Order"}
          >
            <span class="icon order-arrow" class:mirrored={view.activeShelfOrderReverse}>arrow_shape_up_stack</span>
          </button>
        </div>
      {/if}
    {:else}
      {#if showFilterDropdown}
        <div class="v-control-row">
          <div class="v-button-wrapper v-flex-grow">
            <button
              type="button"
              class="v-btn-icon v-sidebar-button v-btn-menu"
              onclick={toggleLibraryFilterMenu}
              class:active={isLibraryFilterMenuOpen}
              title="Filter"
            >
              <span class="icon start-icon">texture</span>
              <span class="v-truncate btn-label">{filterLabel}</span>
              <span class="icon end-icon">{isLibraryFilterMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
            </button>

            {#if isLibraryFilterMenuOpen}
              <div class="v-menu">
                {#each visibleFilters as { key, label } (key)}
                  <button
                    type="button"
                    class="v-menu-item"
                    class:selected={view.activeLibraryFilter === key}
                    onclick={() => {
                      view.setLibraryFilter(key);
                      isLibraryFilterMenuOpen = false;
                    }}
                  >
                    {label}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/if}

      {#if showGroupDropdown}
        <div class="v-control-row">
          <div class="v-button-wrapper v-flex-grow">
            <button
              type="button"
              class="v-btn-icon v-sidebar-button v-btn-menu"
              onclick={toggleGroupMenu}
              class:active={isGroupMenuOpen}
              title="Group By"
            >
              <span class="icon start-icon">stack_group</span>
              <span class="v-truncate btn-label">{groupLabel}</span>
              <span class="icon end-icon">{isGroupMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
            </button>

            {#if isGroupMenuOpen}
              <div class="v-menu">
                {#each visibleGroupers as { key, label } (key)}
                  <button
                    type="button"
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
      {/if}

      {#if showSortDropdown}
        <div class="v-control-row">
          <div class="v-button-wrapper v-flex-grow">
            <button
              type="button"
              class="v-btn-icon v-sidebar-button v-btn-menu"
              onclick={toggleSortMenu}
              class:active={isSortMenuOpen}
              title="Sort By"
            >
              <span class="icon start-icon">swap_vert</span>
              <span class="v-truncate btn-label">{sortLabel}</span>
              <span class="icon end-icon">{isSortMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
            </button>

            {#if isSortMenuOpen}
              <div class="v-menu">
                {#each visibleOrders as { key, label } (key)}
                  <button
                    type="button"
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

          <button
            type="button"
            class="v-btn-icon v-sidebar-button"
            onclick={toggleDirection}
            title={isReverse ? "Reverse Order" : "Default Order"}
          >
            <span class="icon order-arrow" class:mirrored={isReverse}>arrow_shape_up_stack</span>
          </button>
        </div>
      {/if}
    {/if}
  </div>

  <div class="v-sidebar-body">
    <div class="v-sidebar-scroll" bind:this={scrollContainer}>
      <div class="v-scroll-fade-top"></div>
      {#if isCabinetsMode}
        {#each visibleShelves as shelf (shelf.key)}
          <button
            type="button"
            class="v-sidebar-item"
            class:active={activeShelf === shelf.key}
            onclick={() => view.setShelf(shelf.key)}
          >
            <span class="v-truncate label" title={shelf.label}>{shelf.label}</span>
          </button>
        {/each}
      {:else}
        {#each items as item, i (item.value || item.label || i)}
          {@render TreeNode(item, String(i), 0)}
        {/each}
      {/if}
      <div class="scroll-spacer"></div>
      <div class="v-scroll-fade-bottom"></div>
    </div>

    {#if !isCabinetsMode && items.length > 0 && showIndex}
      <SidebarIndex {items} container={scrollContainer} />
    {/if}
  </div>
</div>

<style>
.order-arrow {
  font-size: 22px;
}

.order-arrow.mirrored {
  transform: scaleY(-1);
}

.scroll-spacer {
  height: 12px;
  flex-shrink: 0;
}
</style>
