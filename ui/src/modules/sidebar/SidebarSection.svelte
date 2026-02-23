<script>
  import { library } from "../../library.svelte.js";
  import SidebarItem from "./SidebarItem.svelte";

  let { groupKey } = $props();
  
  let isOpen = $state(false);
  
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
    <img 
      src={isOpen ? "/material/arrow_drop_up_24dp_FFFFFF.svg" : "/material/arrow_drop_down_24dp_FFFFFF.svg"} 
      class="chevron" 
      alt="" 
    />
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
    font-family: var(--font-stack);
    font-size: 14px;
    font-weight: 400;
    cursor: pointer;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .section-header:hover {
    color: var(--text-main);
  }

  .section-header.active {
    color: var(--text-main);
  }

  .chevron {
    width: 20px;
    height: 20px;
    opacity: 0.5;
    flex-shrink: 0;
  }

  .section-header:hover .chevron, .section-header.active .chevron {
    opacity: 1;
  }

  .section-content {
    padding-bottom: 8px;
  }
</style>
