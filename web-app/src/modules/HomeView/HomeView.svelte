<script>
  import { onMount, onDestroy } from "svelte";
  import { library } from "../../library.svelte.js";
  import { nav } from "../../navigation.svelte.js";

  import NavBar from "../NavigationBar/NavBar.svelte";
  import AlbumGrid from "./AlbumGrid/AlbumGrid.svelte";
  import Sidebar from "./Sidebar.svelte";

  let sidebarMode = $state("dynamic");
  let sidebarWidth = $state(160);
  let isResizing = $state(false);

  let isQueueVisible = $derived(nav.activeTab === "queue");
  let isModalVisible = $derived(!!library.focusedAlbum);

  function toggleSidebarMode() {
    sidebarMode = (sidebarMode === "dynamic") ? "static" : "dynamic";
    localStorage.setItem("vellum-sidebar-mode", sidebarMode);
  }

  function handleKeydown(e) {
    if (['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName)) return;
    const key = e.key.toLowerCase();

    if (!isQueueVisible && !isModalVisible) {
      if (key === 's') toggleSidebarMode();
    }
  }

  function startResizing() {
    isResizing = true;
    const move = (e) => { 
      const w = window.innerWidth;
      sidebarWidth = Math.max(140, Math.min(w - e.clientX, 400)); 
    };
    const up = () => {
      isResizing = false;
      localStorage.setItem("vellum-sidebar-width", sidebarWidth);
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }

  onMount(() => {
    const savedLMode = localStorage.getItem("vellum-sidebar-mode");
    if (savedLMode) sidebarMode = savedLMode;
    const savedLWidth = localStorage.getItem("vellum-sidebar-width");
    if (savedLWidth) sidebarWidth = parseInt(savedLWidth);

    window.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="home-view-container" style="--sidebar-width: {sidebarWidth}px;">
  <NavBar />
  
  <div class="workspace">
    <section 
      class="plane home-grid"
      class:offset-layout={sidebarMode === 'static'}
      class:resizing={isResizing}
    >
      <AlbumGrid />
    </section>

    <aside 
      class="sidebar-shell right" 
      class:static={sidebarMode === 'static'} 
      class:dynamic={sidebarMode === 'dynamic'}
      class:dormant={isModalVisible}
    >
      <div class="sidebar-trigger"></div>
      <div class="sidebar-panel">
        <div class="sidebar-resizer" onmousedown={startResizing} role="separator" tabindex="0"></div>
        <div class="sidebar-inner"><Sidebar /></div>
      </div>
    </aside>
  </div>
</div>

<style>
  .home-view-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: row;
    overflow: hidden;
  }

  .workspace {
    flex: 1;
    position: relative;
    height: 100%;
    overflow: hidden;
    min-width: 0;
  }

  .plane {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .home-grid {
    z-index: 1;
    left: 0;
    width: 100%;
    transition: width 0.25s cubic-bezier(0.2, 0, 0, 1);
  }

  .home-grid.offset-layout {
    width: calc(100% - (var(--sidebar-width) - 1px));
  }

  .home-grid.resizing {
    transition: none;
  }

  .sidebar-shell {
    position: fixed;
    top: 0;
    bottom: 0;
    z-index: 100;
    pointer-events: none; 
  }

  .sidebar-shell.dormant {
    pointer-events: none !important; 
  }
  
  .sidebar-shell.right { 
    right: 0; 
    width: var(--sidebar-width); 
    visibility: visible;
  }

  .sidebar-panel {
    position: absolute;
    inset: 0;
    background-color: var(--background-drawer);
    pointer-events: auto; 
    display: flex;
    flex-direction: row;
    transition: transform 0.25s cubic-bezier(0.2, 0, 0, 1);
    box-sizing: border-box;
    box-shadow: var(--album-cover-shadow);
    overflow: visible;
  }

  .sidebar-trigger {
    position: absolute;
    top: 0;
    bottom: 0;
    width: var(--trigger-size);
    z-index: 110;
    pointer-events: auto; 
  }
  .right .sidebar-trigger { right: 0; }

  .sidebar-shell.dynamic.right .sidebar-panel { 
    transform: translateX(100%) translateZ(0); 
    -webkit-backface-visibility: hidden;
    will-change: transform;
  }
  
  .sidebar-shell.dynamic.right:hover .sidebar-panel { 
    transform: translateX(0) translateZ(0); 
  }
  
  .sidebar-shell.static .sidebar-panel { 
    transform: none; 
    -webkit-backface-visibility: visible;
    will-change: auto;
  }

  .sidebar-resizer {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    width: 6px;
    cursor: col-resize;
    z-index: 120;
    transform: translateX(-50%);
    pointer-events: auto;
    background: transparent;
  }

  .sidebar-inner { 
    flex: 1; 
    overflow: hidden; 
  }
</style>
