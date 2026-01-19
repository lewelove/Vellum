<script>
  import { library } from "$state/library.svelte.js";
  import SidebarSection from "./SidebarSection.svelte";
</script>

<div class="sidebar-container">
  <div class="sidebar-header">
    <h1>Library</h1>
    {#if library.activeFilter.key}
      <button class="clear-filter" onclick={() => library.applyFilter(null, null)}>
        Clear Filter
      </button>
    {/if}
  </div>

  <div class="sidebar-scroll">
    <!-- Capabilities Based Rendering -->
    {#each library.capabilities.grouping as groupKey}
      <SidebarSection {groupKey} />
    {/each}
  </div>
</div>

<style>
  .sidebar-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--background-drawer); /* Matches drawer color */
  }

  .sidebar-header {
    padding: 24px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  h1 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-main);
    letter-spacing: -0.02em;
  }

  .clear-filter {
    margin-top: 8px;
    background: none;
    border: 1px solid var(--border-muted);
    color: var(--text-muted);
    padding: 4px 8px;
    font-size: 11px;
    cursor: pointer;
    border-radius: 4px;
  }

  .clear-filter:hover {
    border-color: var(--text-main);
    color: var(--text-main);
  }

  .sidebar-scroll {
    flex: 1;
    overflow-y: auto;
  }
</style>
