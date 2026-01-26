<script>
  import { onMount } from "svelte";
  import { tweened } from "svelte/motion";
  // We'll use a custom cubic-bezier for the 'heavy' feel
  import { cubicOut } from "svelte/easing";
  import { library } from "./library.svelte.js";
  import { nav, setTab } from "./navigation.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  import AlbumGrid from "$modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "$modules/sidebar/Sidebar.svelte";
  import QueueView from "$modules/queue/QueueView.svelte";
  import NavTabs from "$modules/navigation/NavTabs.svelte";

  // Custom "Premium Heavy" Easing
  // This mimics a heavy object with high friction (fast snap, very long tail)
  function premiumHeavy(t) {
    return t === 1 ? 1 : 1 - Math.pow(2, -10 * t); // Similar to expoOut but smoother
  }

  let themeStyles = $derived(getThemeVariables());
  let sidebarMode = $state("dynamic");
  let sidebarWidth = $state(140);
  let isResizing = $state(false);

  const activeIndex = $derived(nav.activeTab === "home" ? 0 : 1);
  
  // Increased duration to 600ms to let the "weight" of the transition be felt
  const pos = tweened(0, { 
    duration: 600, 
    easing: premiumHeavy 
  });

  $effect(() => {
    pos.set(activeIndex);
  });

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
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<main style="{themeStyles} --sidebar-width: {sidebarWidth}px;">
  
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

  <section 
    class="content-viewport"
    class:offset-content={sidebarMode === 'static'}
    class:resizing={isResizing}
  >
    <!-- We use a 3D transform (translate3d) to ensure the GPU handles the "heavy" slide -->
    <div class="view-stage" style="transform: translate3d(-{$pos * 50}%, 0, 0);">
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

  .content-viewport {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    overflow: hidden;
    transition: left 0.25s cubic-bezier(0.2, 0.0, 0.0, 1.0);
    will-change: left;
  }

  .content-viewport.offset-content {
    left: var(--sidebar-width);
  }

  .content-viewport.resizing {
    transition: none;
  }

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
    overflow: hidden;
  }
</style>
