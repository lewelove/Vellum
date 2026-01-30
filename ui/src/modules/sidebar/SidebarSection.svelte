<script>
  import { library } from "../../library.svelte.js";
  import SidebarItem from "./SidebarItem.svelte";

  let { groupKey } = $props();
  
  let isOpen = $state(false);
  
  // Sync Data Loading
  // In the new architecture, data is in RAM, so we don't need async fetching.
  // We derive the items directly from the library state.
  let items = $derived(isOpen ? library.getSidebarGroup(groupKey) : []);

  let hasActiveSelection = $derived(
    library.activeFilter.key && 
    items.some(i => i.filterTarget === library.activeFilter.key && i.value === library.activeFilter.val)
  );

  function toggle() {
    isOpen = !isOpen;
  }

  function formatLabel(key) {
    return key.charAt(0).toUpperCase() + key.slice(1);
  }
</script>

<div class="sidebar-section">
  <button class="section-header" onclick={toggle} class:active={hasActiveSelection}>
    <span class="header-label">{formatLabel(groupKey)}</span>
    <span class="chevron" class:open={isOpen}>›</span>
  </button>

  {#if isOpen}
    <div class="section-content">
        {#each items as item}
          <SidebarItem 
            label={item.label} 
            count={item.count}
            active={library.activeFilter.key === item.filterTarget && library.activeFilter.val === item.value}
            onclick={() => library.applyFilter(item.filterTarget, item.value)}
          />
        {/each}
    </div>
  {/if}
</div>

<style>
  .sidebar-section {
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .section-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 14px;
    font-weight: 400;
    cursor: pointer;
    text-transform: uppercase;
  }

  .section-header:hover {
    color: var(--text-main);
  }

  .section-header.active {
    color: var(--text-main);
  }

  .chevron {
    transform: rotate(0deg);
    transition: transform 0.2s;
    font-size: 16px;
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  .section-content {
    padding-bottom: 8px;
  }
</style>
