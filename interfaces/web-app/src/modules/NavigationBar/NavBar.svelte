<script lang="ts">
  import { nav, setTab } from "../../navigation.svelte.ts";
  import { view } from "../../library/view.svelte.ts";

  let { variant = "solid" } = $props();
</script>

{#snippet NavButton({ icon, tab }: { icon: string, tab: string })}
  <button 
    class="v-btn-icon nav-button" 
    class:active={nav.activeTab === tab} 
    onclick={() => setTab(tab)}
    title={tab}
  >
    <span class="icon">{icon}</span>
  </button>
{/snippet}

{#snippet SubNavButton({ icon, viewId, title }: { icon: string, viewId: "library" | "shelves", title: string })}
  <button 
    class="v-btn-icon nav-button" 
    class:active={view.homeSubView === viewId} 
    onclick={() => {
      view.homeSubView = viewId;
      view.focusedAlbum = null;
      view.refreshView(true);
      view.persistState();
    }}
    {title}
  >
    <span class="icon">{icon}</span>
  </button>
{/snippet}

<nav class="nav-bar" class:v-glass={variant === 'glass'} class:transparent={variant === 'transparent'}>
  <div class="nav-top-section">
    <div class="nav-group top">
      {@render NavButton({ icon: "house", tab: "home" })}
      {@render NavButton({ icon: "queue_music", tab: "queue" })}
    </div>

    {#if nav.activeTab === 'home'}
      <div class="nav-separator"></div>
      <div class="nav-group middle">
        {@render SubNavButton({ icon: "auto_stories", viewId: "library", title: "Library" })}
        {@render SubNavButton({ icon: "newsstand", viewId: "shelves", title: "Shelves" })}
      </div>
    {/if}
  </div>

  <div class="nav-group bottom"></div>
</nav>

<style>
  .nav-bar {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    padding: 8px;
    box-sizing: border-box;
    box-shadow: var(--panel-shadow);
    z-index: 100;
    flex-shrink: 0;
  }

  .nav-bar:not(:global(.v-glass)):not(.transparent) {
    background-color: var(--bg-panel);
  }

  .nav-bar.transparent {
    background-color: transparent;
    box-shadow: none;
    border-right: 1px solid var(--border-muted);
  }

  .nav-top-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    width: 100%;
  }

  .nav-separator {
    width: 100%;
    height: 1px;
    background-color: var(--border-muted);
    margin: 4px 0;
  }
  
  .nav-group {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    width: 100%;
  }

  .nav-button {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    box-shadow: var(--button-shadow-lesser);
    flex-shrink: 0;
    pointer-events: auto;
  }

  .icon {
    font-size: 20px;
  }
</style>
