<script>
  import { onMount } from "svelte";
  import { library } from "./library.svelte.js";
  import { nav, setTab } from "./navigation.svelte.js";
  
  import HomeView from "./modules/HomeView/HomeView.svelte";
  import QueueView from "./modules/QueueView/QueueView.svelte";
  import ModalDrawer from "./modules/HomeView/ModalDrawer/ModalDrawer.svelte";

  let isQueueVisible = $derived(nav.activeTab === "queue");
  let isModalVisible = $derived(!!library.focusedAlbum);

  function handleKeydown(e) {
    if (['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName)) return;
    
    const code = e.code;
    
    if (code === 'Space') {
      e.preventDefault();
      e.stopPropagation();
      fetch('/api/toggle-pause', { method: 'POST' }).catch(() => {});
      return;
    }

    if (code === 'Enter') {
      e.preventDefault();
      e.stopPropagation();
      return;
    }

    const key = e.key.toLowerCase();
    
    if (key === 'escape' && isModalVisible) {
      library.closeFocus();
      return;
    }

    if (
      key === '1' || 
      key === 'h' || 
      key === 'arrowleft'
    ) {
      setTab('home');
    }

    if (
      key === '2' || 
      key === 'l' || 
      key === 'arrowright'
    ) {
      setTab('queue');
    }
  }

  onMount(() => {
    library.init();
    window.addEventListener("keydown", handleKeydown, { capture: true });
    return () => window.removeEventListener("keydown", handleKeydown, { capture: true });
  });
</script>

<svelte:head>
  <link rel="stylesheet" href="/api/theme/css?v={library.themeVersion}" />
</svelte:head>

<main>
  
  <div class="view-layer home">
    <HomeView />
  </div>

  {#if isModalVisible}
    <div class="modal-layer">
        <ModalDrawer album={library.focusedAlbum} onclose={() => library.closeFocus()} />
    </div>
  {/if}

  <div 
    class="view-layer queue"
    class:visible={isQueueVisible}
    aria-hidden={!isQueueVisible}
  >
    <QueueView />
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
  }

  .view-layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: row;
    overflow: hidden;
  }

  .view-layer.home {
    z-index: 1;
  }

  .view-layer.queue {
    z-index: 200;
    background-color: var(--background-main);
    opacity: 0;
    visibility: hidden;
    transition: opacity 0.1s ease-out, visibility 0.1s;
    pointer-events: none;
  }

  .view-layer.queue.visible {
    opacity: 1;
    visibility: visible;
    pointer-events: auto;
  }

  .modal-layer {
    position: absolute;
    inset: 0;
    z-index: 150;
  }
</style>
