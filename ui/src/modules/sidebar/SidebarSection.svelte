<script>
  import { library } from "$state/library.svelte.js";
  import SidebarItem from "./SidebarItem.svelte";

  let { groupKey } = $props();
  
  let isOpen = $state(false);
  let items = $state([]);
  let loading = $state(false);

  // Derived check: Is one of my items currently the active filter?
  let hasActiveSelection = $derived(
    library.activeFilter.key && 
    items.some(i => i.filterTarget === library.activeFilter.key && i.value === library.activeFilter.val)
  );

  async function toggle() {
    isOpen = !isOpen;
    if (isOpen && items.length === 0) {
      loading = true;
      items = await library.fetchSidebarGroup(groupKey);
      loading = false;
    }
  }

  function formatLabel(key) {
    // Basic formatting: "group_genre" -> "Genre"
    // Ideally the backend capabilities would return a display label.
    // For now, capitalize first letter.
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
      {#if loading}
        <div class="loading">Loading...</div>
      {:else}
        {#each items as item}
          <SidebarItem 
            label={item.label} 
            count={item.count}
            active={library.activeFilter.key === item.filterTarget && library.activeFilter.val === item.value}
            onclick={() => library.applyFilter(item.filterTarget, item.value)}
          />
        {/each}
      {/if}
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
    font-weight: 500;
    cursor: pointer;
    text-transform: uppercase;
    letter-spacing: 0.05em;
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

  .loading {
    padding: 8px 16px 8px 32px;
    font-size: 12px;
    color: var(--text-muted);
    font-style: italic;
  }
</style>
