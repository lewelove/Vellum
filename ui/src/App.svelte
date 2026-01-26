<script>
  import { onMount } from "svelte";
  import { library } from "./library.svelte.js";
  import { nav } from "./navigation.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  import AlbumGrid from "$modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "$modules/sidebar/Sidebar.svelte";
  import QueueView from "$modules/queue/QueueView.svelte";
  import NavTabs from "$modules/navigation/NavTabs.svelte";

  let themeStyles = $derived(getThemeVariables());
  
  // State for Sidebar Mode: 'dynamic' (auto-hide) or 'static' (pinned)
  let sidebarMode = $state("dynamic");

  function toggleSidebarMode() {
    sidebarMode = sidebarMode === "dynamic" ? "static" : "dynamic";
    localStorage.setItem("eluxum-sidebar-mode", sidebarMode);
  }

  function handleKeydown(e) {
    if (e.key.toLowerCase() === 's') {
      const tag = document.activeElement?.tagName;
      if (tag !== 'INPUT' && tag !== 'TEXTAREA') {
        toggleSidebarMode();
      }
    }
  }

  onMount(() => {
    library.init();
    
    // Restore preference
    const saved = localStorage.getItem("eluxum-sidebar-mode");
    if (saved === "static" || saved === "dynamic") {
      sidebarMode = saved;
    }

    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<main style={themeStyles}>
  
  <!-- Sidebar Shell -->
  <!-- 
    Groups Nav and Sidebar content. 
    Handles hover detection for "Zen" mode via pointer-events logic.
  -->
  <aside 
    class="sidebar-shell" 
    class:mode-static={sidebarMode === 'static'}
    class:mode-dynamic={sidebarMode === 'dynamic'}
  >
    <!-- Trigger Zone: Invisible strip to catch hover in dynamic mode -->
    <div class="sidebar-trigger"></div>

    <!-- The Sliding Panel -->
    <div class="sidebar-panel">
      <div class="nav-anchor">
        <NavTabs />
      </div>
      <div class="sidebar-inner">
        <Sidebar />
      </div>
    </div>
  </aside>

  <!-- Content Areas -->
  <!-- Adjusted layout based on sidebar mode -->
  {#if nav.activeTab === 'home'}
    <section 
      class="content-pane"
      class:offset-content={sidebarMode === 'static'}
    >
      <AlbumGrid />
    </section>
    
  {:else if nav.activeTab === 'queue'}
    <section class="fullscreen-pane">
      <QueueView />
    </section>
  {/if}
</main>

<style>
  :root {
    --sidebar-width: 140px;
    --trigger-width: 24px;
    --nav-height: 80px;
  }

  main {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: var(--background-main);
  }

  /* --- Sidebar Shell --- */
  
  .sidebar-shell {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: var(--sidebar-width);
    z-index: 100;
    
    /* 
       Magic Logic for Overlay:
       The shell covers the sidebar area, but lets clicks pass through (none).
       Children (trigger, panel) re-enable clicks (auto).
       This allows the shell to maintain :hover state even when mouse moves
       from trigger (24px) to panel (140px).
    */
    pointer-events: none;
  }

  .sidebar-trigger {
    position: absolute;
    top: 0;
    left: 0;
    width: var(--trigger-width);
    height: 100%;
    z-index: 102;
    pointer-events: auto; /* Catch mouse */
    background: transparent;
  }

  .sidebar-panel {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: var(--background-drawer);
    border-right: 1px solid var(--border-muted);
    display: flex;
    flex-direction: column;
    pointer-events: auto; /* Interaction enabled */
    z-index: 101;
    
    transition: transform 0.25s cubic-bezier(0.2, 0.0, 0.0, 1.0);
    will-change: transform;
  }

  /* --- Dynamic Mode Behavior --- */

  .sidebar-shell.mode-dynamic .sidebar-panel {
    transform: translateX(-100%);
    box-shadow: 0 0 15px rgba(0,0,0,0.5); /* Shadow when overlaying */
  }

  /* Reveal on Hover (Shell detects hover on either trigger or panel) */
  .sidebar-shell.mode-dynamic:hover .sidebar-panel {
    transform: translateX(0);
  }

  /* --- Static Mode Behavior --- */

  .sidebar-shell.mode-static .sidebar-panel {
    transform: translateX(0);
    box-shadow: none;
    border-right: 1px solid var(--border-muted);
  }

  .sidebar-shell.mode-static .sidebar-trigger {
    display: none; /* No trigger needed in static */
  }

  /* --- Internal Sidebar Layout --- */

  .nav-anchor {
    flex: 0 0 var(--nav-height);
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .sidebar-inner {
    flex: 1;
    overflow: hidden;
    position: relative;
    /* Reset Sidebar CSS assumptions */
    padding-top: 0 !important; 
  }

  /* --- Content Pane Adaptation --- */

  .content-pane {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0; /* Full width by default (dynamic) */
    overflow: hidden;
    transition: left 0.25s cubic-bezier(0.2, 0.0, 0.0, 1.0);
  }

  .content-pane.offset-content {
    left: var(--sidebar-width);
  }

  .fullscreen-pane {
    position: absolute;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: 5;
  }
</style>
