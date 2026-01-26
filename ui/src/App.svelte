<script>
  import { onMount } from "svelte";
  import { spring } from "svelte/motion";
  import { library } from "./library.svelte.js";
  import { nav, setTab } from "./navigation.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  import AlbumGrid from "$modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "$modules/sidebar/Sidebar.svelte";
  import QueueView from "$modules/queue/QueueView.svelte";
  import QueueTracks from "$modules/queue/QueueTracks.svelte";
  import NavTabs from "$modules/navigation/NavTabs.svelte";

  let themeStyles = $derived(getThemeVariables());
  let sidebarMode = $state("dynamic");
  let sidebarWidth = $state(140);
  let isResizing = $state(false);

  // Queue Sidebar State
  let queueSidebarMode = $state("dynamic");
  let queueSidebarWidth = $state(280);
  let isResizingQueue = $state(false);

  // Viewport Binding for Absolute Pixel Calculation
  let viewportWidth = $state(0);

  // index: 0 = Home, 1 = Queue
  const activeIndex = $derived(nav.activeTab === "home" ? 0 : 1);

  // Derived state for Queue Sidebar visibility
  let isQueueSidebarActive = $derived(nav.activeTab === 'queue');
  
  /**
   * Physics-based Spring Store
   * stiffness: - Lower values increase the feeling of "mass"
   * damping: - Higher values (closer to 1.0) prevent bounce and create a "premium oil-damped" feel
   * precision: - The Snap Threshold. Animation settles instantly when distance < 0.1px
   */
  const pos = spring(0, {
    stiffness: 0.05,
    damping: 0.5,
    precision: 0.01 
  });

  $effect(() => {
    // 1. Calculate Ideal Geometric Target
    const rawTarget = activeIndex * viewportWidth;

    // 2. Hardware-Grid Snapping (DPR)
    // We pre-snap the target so the spring settles into a pixel-perfect slot
    const dpr = window.devicePixelRatio || 1;
    const snappedTarget = Math.round(rawTarget * dpr) / dpr;

    // 3. Update the Spring
    pos.set(snappedTarget);
  });

  function toggleSidebarMode() {
    sidebarMode = sidebarMode === "dynamic" ? "static" : "dynamic";
    localStorage.setItem("eluxum-sidebar-mode", sidebarMode);
  }

  function toggleQueueSidebarMode() {
    queueSidebarMode = queueSidebarMode === "dynamic" ? "static" : "dynamic";
    localStorage.setItem("eluxum-queue-sidebar-mode", queueSidebarMode);
  }

  function handleKeydown(e) {
    const tag = document.activeElement?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA') return;
    const key = e.key.toLowerCase();
    
    if (key === 's') {
      toggleSidebarMode();
    } else if (key === 'q') {
      toggleQueueSidebarMode();
    } else if (key === '1' || key === 'h' || key === 'arrowleft') {
      setTab('home');
    } else if (key === '2' || key === 'l' || key === 'arrowright') {
      setTab('queue');
    }
  }

  // Left Sidebar Resizing
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

  // Right Queue Sidebar Resizing
  function startResizingQueue(e) {
    e.preventDefault();
    isResizingQueue = true;
    window.addEventListener("mousemove", handleMouseMoveQueue);
    window.addEventListener("mouseup", stopResizingQueue);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }

  function handleMouseMoveQueue(e) {
    if (!isResizingQueue) return;
    // Calculate width from right edge
    const newWidth = window.innerWidth - e.clientX;
    queueSidebarWidth = Math.max(200, Math.min(newWidth, 600));
  }

  function stopResizingQueue() {
    isResizingQueue = false;
    window.removeEventListener("mousemove", handleMouseMoveQueue);
    window.removeEventListener("mouseup", stopResizingQueue);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    localStorage.setItem("eluxum-queue-sidebar-width", queueSidebarWidth.toString());
  }

  onMount(() => {
    library.init();
    
    const savedMode = localStorage.getItem("eluxum-sidebar-mode");
    if (savedMode === "static" || savedMode === "dynamic") {
      sidebarMode = savedMode;
    }
    const savedWidth = localStorage.getItem("eluxum-sidebar-width");
    if (savedWidth) {
      sidebarWidth = parseInt(savedWidth);
    }

    const savedQMode = localStorage.getItem("eluxum-queue-sidebar-mode");
    if (savedQMode === "static" || savedQMode === "dynamic") {
      queueSidebarMode = savedQMode;
    }
    const savedQWidth = localStorage.getItem("eluxum-queue-sidebar-width");
    if (savedQWidth) {
      queueSidebarWidth = parseInt(savedQWidth);
    }

    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<main style="{themeStyles} --sidebar-width: {sidebarWidth}px; --queue-sidebar-width: {queueSidebarWidth}px;">
  
  <aside 
    class="sidebar-shell" 
    class:mode-static={sidebarMode === 'static'}
    class:mode-dynamic={sidebarMode === 'dynamic'}
    class:resizing={isResizing}
  >
    <div class="sidebar-trigger"></div>
    <div class="sidebar-panel">
      <div class="nav-anchor">
        <NavTabs />
      </div>
      <div class="sidebar-inner">
        <Sidebar />
      </div>
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

  <aside 
    class="sidebar-shell right-shell" 
    class:mode-static={queueSidebarMode === 'static'}
    class:mode-dynamic={queueSidebarMode === 'dynamic'}
    class:active={isQueueSidebarActive}
    class:resizing={isResizingQueue}
  >
    <div class="sidebar-panel right-panel">
      <div class="sidebar-inner">
        <QueueTracks isVisible={isQueueSidebarActive} />
      </div>
      <div 
        class="sidebar-resizer left-resizer" 
        onmousedown={startResizingQueue}
        role="separator"
        aria-orientation="vertical"
        aria-valuenow={queueSidebarWidth}
        tabindex="-1"
      ></div>
    </div>
  </aside>

  <section 
    class="content-viewport"
    bind:clientWidth={viewportWidth}
    class:offset-content-left={sidebarMode === 'static'}
    class:offset-content-right={isQueueSidebarActive && queueSidebarMode === 'static'}
    class:resizing={isResizing || isResizingQueue}
  >
    <div 
      class="view-stage" 
      style="transform: translate3d(-{$pos}px, 0, 0);"
    >
      <div class="view-page">
        <AlbumGrid />
      </div>
      <div class="view-page">
        <QueueView />
      </div>
    </div>
  </section>

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

  /* --- LEFT SIDEBAR --- */

  .sidebar-shell {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: var(--sidebar-width);
    z-index: 100;
    pointer-events: none;
  }

  .sidebar-trigger {
    position: absolute;
    top: 0;
    left: 0;
    width: var(--trigger-width);
    height: 100%;
    z-index: 102;
    pointer-events: auto;
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
    pointer-events: auto;
    z-index: 101;
    transition: transform 0.25s cubic-bezier(0.2, 0.0, 0.0, 1.0);
    will-change: transform;
  }

  .sidebar-shell.resizing .sidebar-panel {
    transition: none;
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

  .sidebar-shell.mode-dynamic .sidebar-panel {
    transform: translateX(-100%);
    box-shadow: 0 0 15px rgba(0,0,0,0.5);
  }

  .sidebar-shell.mode-dynamic:hover .sidebar-panel {
    transform: translateX(0);
  }

  .sidebar-shell.mode-static .sidebar-panel {
    transform: translateX(0);
    box-shadow: none;
    border-right: 1px solid var(--border-muted);
  }

  .sidebar-shell.mode-static .sidebar-trigger {
    display: none;
  }

  /* --- RIGHT SIDEBAR (QUEUE) --- */

  .right-shell {
    left: auto;
    right: 0;
    width: var(--queue-sidebar-width);
  }

  .right-panel {
    left: auto;
    right: 0;
    border-right: none;
    border-left: 1px solid var(--border-muted);
  }

  .left-resizer {
    right: auto;
    left: -3px;
  }

  /* 
     Fix for Directionality:
     The generic .sidebar-shell.mode-dynamic pulls it to -100% (Left).
     We must explicitly force the Right Shell to +100% (Right) when hidden.
  */

  .right-shell .sidebar-panel,
  .right-shell.mode-dynamic .sidebar-panel {
    transform: translateX(100%);
  }

  /* Active States */

  .right-shell.active.mode-static .sidebar-panel {
    transform: translateX(0);
    box-shadow: none;
  }

  .right-shell.active.mode-dynamic .sidebar-panel {
    transform: translateX(0);
    box-shadow: 0 0 15px rgba(0,0,0,0.5);
    border-left: 1px solid rgba(255, 255, 255, 0.1); 
  }

  /* --- COMMON INNER --- */

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
    padding-top: 0 !important; 
  }

  /* --- VIEWPORT --- */

  .content-viewport {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    overflow: hidden;
    transition: left 0.25s cubic-bezier(0.2, 0.0, 0.0, 1.0), right 0.25s cubic-bezier(0.2, 0.0, 0.0, 1.0);
    will-change: left, right;
  }

  .content-viewport.offset-content-left {
    left: var(--sidebar-width);
  }

  .content-viewport.offset-content-right {
    right: var(--queue-sidebar-width);
  }

  .content-viewport.resizing {
    transition: none;
  }

  .view-stage {
    display: flex;
    width: 200%;
    height: 100%;
    will-change: transform;
    backface-visibility: hidden;
  }

  .view-page {
    width: 50%;
    height: 100%;
    flex-shrink: 0;
    position: relative;
    overflow: hidden;
  }
</style>
