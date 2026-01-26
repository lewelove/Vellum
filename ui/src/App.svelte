<script>
  import { onMount } from "svelte";
  import { library } from "./library.svelte.js";
  import { nav, setTab } from "./navigation.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  
  // Component Imports
  import AlbumGrid from "./modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "./modules/sidebar/Sidebar.svelte";
  import QueueView from "./modules/queue/QueueView.svelte";
  import NavTabs from "./modules/navigation/NavTabs.svelte";

  // Reactive Theme Variables
  let themeStyles = $derived(getThemeVariables());
  
  // Sidebar State
  let sidebarMode = $state("dynamic");
  let sidebarWidth = $state(160);
  let isResizingLeft = $state(false);

  let viewportWidth = $state(0);
  let isQueueVisible = $derived(nav.activeTab === "queue");

  // Define the offset variable for children to compensate for the shift
  let sidebarOffset = $derived(sidebarMode === 'static' ? sidebarWidth : 0);

  function toggleSidebarMode() {
    sidebarMode = (sidebarMode === "dynamic") ? "static" : "dynamic";
    localStorage.setItem("eluxum-sidebar-mode", sidebarMode);
  }

  function handleKeydown(e) {
    if (['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName)) return;
    const key = e.key.toLowerCase();
    if (key === 's') toggleSidebarMode();
    if (key === '1' || key === 'h' || key === 'arrowleft') setTab('home');
    if (key === '2' || key === 'l' || key === 'arrowright') setTab('queue');
  }

  function startResizingLeft() {
    isResizingLeft = true;
    const move = (e) => { sidebarWidth = Math.max(140, Math.min(e.clientX, 400)); };
    const up = () => {
      isResizingLeft = false;
      localStorage.setItem("eluxum-sidebar-width", sidebarWidth);
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }

  onMount(() => {
    library.init();
    
    const savedLMode = localStorage.getItem("eluxum-sidebar-mode");
    if (savedLMode) sidebarMode = savedLMode;
    const savedLWidth = localStorage.getItem("eluxum-sidebar-width");
    if (savedLWidth) sidebarWidth = parseInt(savedLWidth);

    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<main style="{themeStyles} --sidebar-width: {sidebarWidth}px; --sidebar-offset: {sidebarOffset}px;">
  
  <section 
    class="content-viewport"
    bind:clientWidth={viewportWidth}
    class:offset-left={sidebarMode === 'static'}
    class:resizing={isResizingLeft}
  >
    <!-- Multi-surface stack -->
    <div class="view-stack">
      <!-- Base Layer: Album Grid -->
      <div class="view-page grid-layer">
        <AlbumGrid />
      </div>

      <!-- Overlay Layer: Queue View -->
      <div 
        class="view-page queue-layer" 
        class:visible={isQueueVisible}
      >
        <QueueView />
      </div>
    </div>
  </section>

  <aside class="sidebar-shell left" class:static={sidebarMode === 'static'} class:dynamic={sidebarMode === 'dynamic'}>
    <div class="sidebar-trigger"></div>
    <div class="sidebar-panel">
      <div class="nav-anchor"><NavTabs /></div>
      <div class="sidebar-inner"><Sidebar /></div>
      <div class="sidebar-resizer" onmousedown={startResizingLeft}></div>
    </div>
  </aside>

</main>

<style>
  :root {
    --nav-height: 80px;
    --trigger-size: 24px;
  }

  main {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background-color: var(--background-main);
  }

  .content-viewport {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    width: 100%;
    overflow: hidden;
    z-index: 10;
    transition: left 0.25s cubic-bezier(0.2, 0, 0, 1), width 0.25s cubic-bezier(0.2, 0, 0, 1);
  }

  .content-viewport.offset-left { 
    left: var(--sidebar-width); 
    width: calc(100% - var(--sidebar-width));
  }

  .content-viewport.resizing { transition: none; }

  .view-stack {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .view-page {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .grid-layer {
    z-index: 1;
  }

  .queue-layer {
    z-index: 2;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.1s ease-out;
  }

  .queue-layer.visible {
    opacity: 1;
    pointer-events: auto;
  }

  .sidebar-shell {
    position: fixed;
    top: 0;
    bottom: 0;
    z-index: 100;
    pointer-events: none;
  }
  
  .sidebar-shell.left { 
    left: 0; 
    width: var(--sidebar-width); 
    visibility: visible;
    pointer-events: none;
  }

  .sidebar-panel {
    position: absolute;
    inset: 0;
    background-color: var(--background-drawer);
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    transition: transform 0.25s cubic-bezier(0.2, 0, 0, 1);
  }

  .sidebar-trigger {
    position: absolute;
    top: 0;
    bottom: 0;
    width: var(--trigger-size);
    z-index: 110;
    pointer-events: auto;
  }
  .left .sidebar-trigger { left: 0; }

  .sidebar-shell.dynamic.left .sidebar-panel { transform: translateX(-100%); }
  .sidebar-shell.dynamic.left:hover .sidebar-panel { transform: translateX(0); }
  
  .sidebar-shell.static .sidebar-panel { transform: translateX(0); box-shadow: none; }
  .sidebar-shell.static .sidebar-trigger { display: none; }

  .left .sidebar-panel { border-right: 1px solid var(--border-muted); }

  .sidebar-resizer {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
    z-index: 120;
  }
  .left .sidebar-resizer { right: -3px; }

  .nav-anchor { height: var(--nav-height); display: flex; align-items: center; justify-content: center; }
  .sidebar-inner { flex: 1; overflow: hidden; }
</style>
