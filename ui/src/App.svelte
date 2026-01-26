<script>
  import { onMount } from "svelte";
  import { library } from "./library.svelte.js";
  import { nav, setTab } from "./navigation.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  import AlbumGrid from "$modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "$modules/sidebar/Sidebar.svelte";
  import QueueView from "$modules/queue/QueueView.svelte";
  import NavTabs from "$modules/navigation/NavTabs.svelte";

  let themeStyles = $derived(getThemeVariables());
  
  // State for Sidebar Mode: 'dynamic' (auto-hide) or 'static' (pinned)
  let sidebarMode = $state("dynamic");

  // Resizing State
  let sidebarWidth = $state(140);
  let isResizing = $state(false);

  function toggleSidebarMode() {
    sidebarMode = sidebarMode === "dynamic" ? "static" : "dynamic";
    localStorage.setItem("eluxum-sidebar-mode", sidebarMode);
  }

  function handleKeydown(e) {
    const tag = document.activeElement?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA') return;

    const key = e.key.toLowerCase();

    if (key === 's') {
      toggleSidebarMode();
    } else if (key === '1') {
      setTab('home');
    } else if (key === '2') {
      setTab('queue');
    }
  }

  // Resizing Logic
  function startResizing(e) {
    e.preventDefault();
    isResizing = true;
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", stopResizing);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }

  function handleMouseMove(e) {
    if (!isResizing) return;
    // Clamp width between 140px and 400px
    sidebarWidth = Math.max(140, Math.min(e.clientX, 400));
  }

  function stopResizing() {
    isResizing = false;
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", stopResizing);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    localStorage.setItem("eluxum-sidebar-width", sidebarWidth.toString());
  }

  onMount(() => {
    library.init();
    
    // Restore preference
    const savedMode = localStorage.getItem("eluxum-sidebar-mode");
    if (savedMode === "static" || savedMode === "dynamic") {
      sidebarMode = savedMode;
    }

    const savedWidth = localStorage.getItem("eluxum-sidebar-width");
    if (savedWidth) {
      sidebarWidth = parseInt(savedWidth);
    }

    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<main style="{themeStyles} --sidebar-width: {sidebarWidth}px;">
  
  <!-- Sidebar Shell -->
  <!-- 
    Groups Nav and Sidebar content. 
    Handles hover detection for "Zen" mode via pointer-events logic.
  -->
  <aside 
    class="sidebar-shell" 
    class:mode-static={sidebarMode === 'static'}
    class:mode-dynamic={sidebarMode === 'dynamic'}
    class:resizing={isResizing}
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

      <!-- Resize Handle -->
      <div 
        class="sidebar-resizer" 
        onmousedown={startResizing}
        role="separator"
        aria-orientation="vertical"
        aria-valuenow={sidebarWidth}
        tabindex="-1"
      ></div>
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

  .sidebar-shell.resizing .sidebar-panel {
    transition: none; /* Disable transition during active drag */
  }

  .sidebar-resizer {
    position: absolute;
    top: 0;
    right: -3px;
    width: 6px;
    height: 100%;
    cursor: col-resize;
    z-index: 110;
    background: transparent;
  }

  .sidebar-resizer:hover {
    background: rgba(255, 255, 255, 0.05);
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
    will-change: left;
  }

  .sidebar-shell.resizing ~ .content-pane {
    transition: none; /* Disable transition during active drag */
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
