<script>
  import { onMount } from "svelte";
  import { library } from "./library.svelte.js";
  import { nav, setTab } from "./navigation.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  
  import AlbumGrid from "./modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "./modules/sidebar/Sidebar.svelte";
  import QueueView from "./modules/queue/QueueView.svelte";
  import ModalDrawer from "./modules/album-grid/ModalDrawer.svelte";
  import NavBar from "./modules/navigation/NavBar.svelte";

  let themeStyles = $derived(getThemeVariables());
  
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
    
    if (key === 'escape' && isModalVisible) {
      library.closeFocus();
      return;
    }

    if (!isQueueVisible && !isModalVisible) {
      if (key === 's') toggleSidebarMode();
    }

    if (key === '1' || key === 'h' || key === 'arrowleft') setTab('home');
    if (key === '2' || key === 'l' || key === 'arrowright') setTab('queue');
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
    library.init();
    
    const savedLMode = localStorage.getItem("vellum-sidebar-mode");
    if (savedLMode) sidebarMode = savedLMode;
    const savedLWidth = localStorage.getItem("vellum-sidebar-width");
    if (savedLWidth) sidebarWidth = parseInt(savedLWidth);

    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<main style="{themeStyles} --sidebar-width: {sidebarWidth}px;">
  
  <NavBar />

  <div class="workspace">
    <section 
      class="plane home-layer"
      class:offset-layout={sidebarMode === 'static'}
      class:resizing={isResizing}
      aria-hidden={isQueueVisible}
    >
      <AlbumGrid />
    </section>

    <aside 
      class="sidebar-shell right" 
      class:static={sidebarMode === 'static'} 
      class:dynamic={sidebarMode === 'dynamic'}
      class:dormant={isQueueVisible || isModalVisible}
    >
      <div class="sidebar-trigger"></div>
      <div class="sidebar-panel">
        <div class="sidebar-resizer" onmousedown={startResizing}></div>
        <div class="sidebar-inner"><Sidebar /></div>
      </div>
    </aside>

    {#if isModalVisible}
      <div class="modal-layer">
          <ModalDrawer album={library.focusedAlbum} onclose={() => library.closeFocus()} />
      </div>
    {/if}

    <section 
      class="plane queue-layer"
      class:visible={isQueueVisible}
      aria-hidden={!isQueueVisible}
    >
      <QueueView />
    </section>
  </div>

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
    display: flex;
    flex-direction: row;
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

  .home-layer {
    z-index: 1;
    left: 0;
    width: 100%;
    transition: width 0.25s cubic-bezier(0.2, 0, 0, 1);
  }

  .home-layer.offset-layout {
    /* When sidebar is static on right, reduce width from the right side */
    width: calc(100% - (var(--sidebar-width) - 1px));
  }

  .home-layer.resizing {
    transition: none;
  }

  .queue-layer {
    z-index: 200; 
    background-color: var(--background-drawer);
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transition: opacity 0.1s ease-out, visibility 0.1s; 
  }

  .queue-layer.visible {
    opacity: 1;
    visibility: visible;
    pointer-events: auto;
  }

  .modal-layer {
    position: absolute;
    inset: 0;
    z-index: 150;
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
    flex-direction: row; /* Changed to row to accomodate resizer on left */
    transition: transform 0.25s cubic-bezier(0.2, 0, 0, 1);
    box-sizing: border-box;
    box-shadow: var(--album-cover-shadow);
    overflow: hidden;
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
    width: 6px;
    height: 100%;
    cursor: col-resize;
    z-index: 120;
    flex-shrink: 0;
    position: relative;
    /* Visual hint for resizer could go here */
  }

  .sidebar-inner { flex: 1; overflow: hidden; }
</style>
