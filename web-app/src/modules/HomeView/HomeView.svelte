<script>
  import { onMount } from "svelte";
  import { library } from "../../library.svelte.js";

  import NavBar from "../NavigationBar/NavBar.svelte";
  import AlbumGrid from "./AlbumGrid/AlbumGrid.svelte";
  import Sidebar from "./Sidebar.svelte";

  let sidebarWidth = $state(160);
  let isResizing = $state(false);

  let isModalVisible = $derived(!!library.focusedAlbum);

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
    const savedLWidth = localStorage.getItem("vellum-sidebar-width");
    if (savedLWidth) sidebarWidth = parseInt(savedLWidth);
  });
</script>

<div class="home-view-container" style="--sidebar-width: {sidebarWidth}px;">
  <NavBar />
  
  <div class="workspace">
    <section 
      class="plane home-grid"
      class:resizing={isResizing}
    >
      <AlbumGrid />
    </section>

    <aside 
      class="sidebar-shell right" 
      class:dormant={isModalVisible}
    >
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
    width: calc(100% - (var(--sidebar-width) - 1px));
    transition: width 0.25s cubic-bezier(0.2, 0, 0, 1);
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
    box-sizing: border-box;
    box-shadow: var(--panel-shadow);
    overflow: visible;
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
