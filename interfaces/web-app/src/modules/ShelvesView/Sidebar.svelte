<script lang="ts">
import { view } from "../../library/view.svelte.ts";
import { collection } from "../../library/collection.svelte.ts";

let isCabinetMenuOpen = $state(false);
let isOrderMenuOpen = $state(false);

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

let visibleOrders = $derived(collection.getVisibleOrdersForCabinet(activeCabinet));
let showOrderDropdown = $derived(visibleOrders.length >= 1);

let orderLabel = $derived(
  view.activeShelfOrder === "original"
    ? "Original"
    : collection.availableOrders[view.activeShelfOrder]?.label || "Original"
);

function selectCabinet(key: string) {
  view.setCabinet(key);
  isCabinetMenuOpen = false;
}

function selectShelf(key: string) {
  view.setShelf(key);
}

function selectOrder(key: string) {
  view.setShelfOrder(key);
  isOrderMenuOpen = false;
}

function toggleDirection() {
  view.toggleShelfOrderDirection();
}

function toggleSubView() {
  view.homeSubView = view.homeSubView === "library" ? "shelves" : "library";
  view.focusedAlbum = null;
  view.refreshView(true);
  view.persistState();
}

function toggleCabinetMenu() {
  isCabinetMenuOpen = !isCabinetMenuOpen;
  if (isCabinetMenuOpen) {
    isOrderMenuOpen = false;
  }
}

function toggleOrderMenu() {
  isOrderMenuOpen = !isOrderMenuOpen;
  if (isOrderMenuOpen) {
    isCabinetMenuOpen = false;
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    isCabinetMenuOpen = false;
    isOrderMenuOpen = false;
  }
}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="v-sidebar-container">
  <div class="v-sidebar-controls">
    <div class="v-control-row">
      <button
        type="button"
        class="v-btn-icon v-sidebar-button"
        onclick={toggleSubView}
        title={view.homeSubView === "library" ? "Libraries" : "Shelves"}
      >
        <span class="icon">{view.homeSubView === "library" ? "auto_stories" : "shelves"}</span>
      </button>

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
    </div>

    {#if showOrderDropdown}
      <div class="v-control-row">
        <div class="v-button-wrapper v-flex-grow">
          <button
            type="button"
            class="v-btn-icon v-sidebar-button v-btn-menu"
            onclick={toggleOrderMenu}
            class:active={isOrderMenuOpen}
            title="Sort By"
          >
            <span class="icon start-icon">swap_vert</span>
            <span class="v-truncate btn-label">{orderLabel}</span>
            <span class="icon end-icon">{isOrderMenuOpen ? "arrow_drop_up" : "arrow_drop_down"}</span>
          </button>

          {#if isOrderMenuOpen}
            <div class="v-menu">
              <button
                type="button"
                class="v-menu-item"
                class:selected={view.activeShelfOrder === "original"}
                onclick={() => selectOrder("original")}
              >
                Original
              </button>
              {#each visibleOrders as { key, label } (key)}
                <button
                  type="button"
                  class="v-menu-item"
                  class:selected={view.activeShelfOrder === key}
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
          title={view.activeShelfOrderReverse ? "Reverse Order" : "Default Order"}
        >
          <span class="icon order-arrow" class:mirrored={view.activeShelfOrderReverse}>arrow_shape_up_stack</span>
        </button>
      </div>
    {/if}
  </div>

  <div class="v-sidebar-scroll">
    <div class="v-scroll-fade-top"></div>
    {#each visibleShelves as shelf (shelf.key)}
      <button
        type="button"
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
