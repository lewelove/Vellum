<script>
  import { onMount } from "svelte";
  import { spring } from "svelte/motion";
  import { library } from "./library.svelte.js";
  import { nav, setTab } from "./navigation.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  
  // Component Imports
  import AlbumGrid from "./modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "./modules/sidebar/Sidebar.svelte";
  import QueueView from "./modules/queue/QueueView.svelte";
  import QueueTracks from "./modules/queue/QueueTracks.svelte";
  import NavTabs from "./modules/navigation/NavTabs.svelte";

  // Reactive Theme Variables
  let themeStyles = $derived(getThemeVariables());
  
  // Safe State Initialization (Prevents crashes if localStorage is empty)
  let sidebarMode = $state("dynamic");
  let sidebarWidth = $state(160);
  let isResizingLeft = $state(false);

  let queueSidebarMode = $state("dynamic");
  let queueSidebarWidth = $state(280);
  let isResizingRight = $state(false);
  
  let isRightHovered = $state(false);
  let isNavSettled = $state(true);
  let viewportWidth = $state(0);

  // Derive target page index
  const activeIndex = $derived(nav.activeTab === "home" ? 0 : 1);
  
  // Derive Queue Sidebar state
  const isQueueSidebarActive = $derived(
    isRightHovered ||
    queueSidebarMode === 'static' ||
    (nav.activeTab === 'queue' && isNavSettled)
  );
  
  // Paging Physics
  const pos = spring(0, { stiffness: 0.08, damping: 0.6 });

  // Paging Effect: Sync spring to viewport width and active tab
  $effect(() => {
    if (viewportWidth > 0) {
      const target = activeIndex * viewportWidth;
      const dpr = window.devicePixelRatio || 1;
      const snapped = Math.round(target * dpr) / dpr;
      pos.set(snapped);
      
      isNavSettled = false;
      const timer = setTimeout(() => { isNavSettled = true; }, 450);
      return () => clearTimeout(timer);
    }
  });

  function toggleSidebarMode() {
    sidebarMode = (sidebarMode === "dynamic") ? "static" : "dynamic";
    localStorage.setItem("eluxum-sidebar-mode", sidebarMode);
  }

  function toggleQueueSidebarMode() {
    queueSidebarMode = (queueSidebarMode === "dynamic") ? "static" : "dynamic";
    localStorage.setItem("eluxum-queue-sidebar-mode", queueSidebarMode);
  }

  function handleKeydown(e) {
    if (['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName)) return;
    const key = e.key.toLowerCase();
    if (key === 's') toggleSidebarMode();
    if (key === 'q') toggleQueueSidebarMode();
    if (key === '1' || key === 'h' || key === 'arrowleft') setTab('home');
    if (key === '2' || key === 'l' || key === 'arrowright') setTab('queue');
  }

  // Left Sidebar Resizing Logic
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

  // Right Sidebar Resizing Logic
  function startResizingRight() {
    isResizingRight = true;
    const move = (e) => { queueSidebarWidth = Math.max(200, Math.min(window.innerWidth - e.clientX, 600)); };
    const up = () => {
      isResizingRight = false;
      localStorage.setItem("eluxum-queue-sidebar-width", queueSidebarWidth);
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }

  onMount(() => {
    library.init();
    
    // Load persisted settings
    const savedLMode = localStorage.getItem("eluxum-sidebar-mode");
    if (savedLMode) sidebarMode = savedLMode;
    const savedLWidth = localStorage.getItem("eluxum-sidebar-width");
    if (savedLWidth) sidebarWidth = parseInt(savedLWidth);

    const savedRMode = localStorage.getItem("eluxum-queue-sidebar-mode");
    if (savedRMode) queueSidebarMode = savedRMode;
    const savedRWidth = localStorage.getItem("eluxum-queue-sidebar-width");
    if (savedRWidth) queueSidebarWidth = parseInt(savedRWidth);

    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<main 
  style="{themeStyles} --sidebar-width: {sidebarWidth}px; --queue-sidebar-width: {queueSidebarWidth}px;"
>
  
  <section 
    class="content-viewport"
    bind:clientWidth={viewportWidth}
    class:offset-left={sidebarMode === 'static'}
    class:offset-right={nav.activeTab === 'home' && queueSidebarMode === 'static'}
    class:resizing={isResizingLeft || isResizingRight}
  >
    <div class="view-stage" style="transform: translate3d(-{$pos}px, 0, 0);">
      <div class="view-page">
        <AlbumGrid />
      </div>
      <div class="view-page">
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

  <aside class="sidebar-shell right" 
    class:static={queueSidebarMode === 'static'} 
    class:dynamic={queueSidebarMode === 'dynamic'}
    class:active={isQueueSidebarActive}
    onmouseenter={() => isRightHovered = true}
    onmouseleave={() => isRightHovered = false}
  >
    <div class="sidebar-trigger right-trigger"></div>
    <div class="sidebar-panel">
      <div class="sidebar-inner"><QueueTracks isVisible={isQueueSidebarActive} /></div>
      <div class="sidebar-resizer right-resizer" onmousedown={startResizingRight}></div>
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
    inset: 0;
    overflow: hidden;
    z-index: 10;
    transition: left 0.25s cubic-bezier(0.2, 0, 0, 1), right 0.25s cubic-bezier(0.2, 0, 0, 1);
  }
  .content-viewport.offset-left { left: var(--sidebar-width); }
  .content-viewport.offset-right { right: var(--queue-sidebar-width); }
  .content-viewport.resizing { transition: none; }

  .view-stage {
    display: flex;
    width: 200%;
    height: 100%;
    will-change: transform;
  }
  .view-page {
    width: 50%;
    height: 100%;
    flex-shrink: 0;
    position: relative;
  }

  .sidebar-shell {
    position: fixed;
    top: 0;
    bottom: 0;
    z-index: 100;
    pointer-events: none;
  }
  .sidebar-shell.left { left: 0; width: var(--sidebar-width); }
  .sidebar-shell.right { right: 0; width: var(--queue-sidebar-width); }

  .sidebar-panel {
    position: absolute;
    inset: 0;
    background-color: var(--background-drawer);
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    transition: transform 0.25s cubic-bezier(0.2, 0, 0, 1);
    box-shadow: 0 0 20px rgba(0,0,0,0.4);
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
  .right .sidebar-trigger { right: -2px; width: 26px; }

  .sidebar-shell.dynamic.left .sidebar-panel { transform: translateX(-100%); }
  .sidebar-shell.dynamic.left:hover .sidebar-panel { transform: translateX(0); }
  
  .sidebar-shell.dynamic.right .sidebar-panel { transform: translateX(100%); }
  .sidebar-shell.dynamic.right:hover .sidebar-panel,
  .sidebar-shell.dynamic.right.active .sidebar-panel { transform: translateX(0); }

  .sidebar-shell.static .sidebar-panel { transform: translateX(0); box-shadow: none; }
  .sidebar-shell.static .sidebar-trigger { display: none; }

  .left .sidebar-panel { border-right: 1px solid var(--border-muted); }
  .right .sidebar-panel { border-left: 1px solid var(--border-muted); }

  .sidebar-resizer {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
    z-index: 120;
  }
  .left .sidebar-resizer { right: -3px; }
  .right .sidebar-resizer { left: -3px; }

  .nav-anchor { height: var(--nav-height); display: flex; align-items: center; justify-content: center; }
  .sidebar-inner { flex: 1; overflow: hidden; }
</style>
