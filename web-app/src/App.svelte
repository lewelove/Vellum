<script>
  import { onMount } from "svelte";
  import { library } from "./library.svelte.js";
  import { nav, setTab } from "./navigation.svelte.js";
  
  import HomeView from "./modules/HomeView/HomeView.svelte";
  import ShelvesView from "./modules/ShelvesView/ShelvesView.svelte";
  import QueueView from "./modules/QueueView/QueueView.svelte";
  import ModalDrawer from "./modules/HomeView/ModalDrawer/ModalDrawer.svelte";

  const tabOrder = { home: 1, queue: 2, shelves: 3 };
  let currentTab = $state(nav.activeTab);
  let retentionTab = $state(null);
  let instantTab = $state(null);

  $effect(() => {
    if (nav.activeTab !== currentTab) {
      const oldTab = currentTab;
      const newTab = nav.activeTab;
      
      if (tabOrder[newTab] > tabOrder[oldTab]) {
        retentionTab = oldTab;
        instantTab = null;
        setTimeout(() => {
          if (retentionTab === oldTab) retentionTab = null;
        }, 100);
      } else {
        retentionTab = null;
        instantTab = newTab;
        setTimeout(() => {
          if (instantTab === newTab) instantTab = null;
        }, 100);
      }
      
      currentTab = newTab;
    }
  });

  let isHomeActive = $derived(currentTab === 'home');
  let isQueueActive = $derived(currentTab === 'queue');
  let isShelvesActive = $derived(currentTab === 'shelves');

  let isHomeVisible = true;
  let isQueueVisible = $derived(currentTab === 'queue' || retentionTab === 'queue');
  let isShelvesVisible = $derived(currentTab === 'shelves' || retentionTab === 'shelves');

  let isQueueInstant = $derived(instantTab === 'queue');
  let isShelvesInstant = $derived(instantTab === 'shelves');
  
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

    if (key === '1' || key === 'h') setTab('home');
    if (key === '2' || key === 'q') setTab('queue');
    if (key === '3' || key === 's') setTab('shelves');
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
  
  <div class="view-layer home" class:visible={isHomeVisible} class:active={isHomeActive} aria-hidden={!isHomeActive}>
    <HomeView />
  </div>

  <div class="view-layer queue" class:visible={isQueueVisible} class:active={isQueueActive} class:instant={isQueueInstant} aria-hidden={!isQueueActive}>
    <QueueView />
  </div>

  <div class="view-layer shelves" class:visible={isShelvesVisible} class:active={isShelvesActive} class:instant={isShelvesInstant} aria-hidden={!isShelvesActive}>
    <ShelvesView />
  </div>

  {#if isModalVisible}
    <div class="modal-layer">
        <ModalDrawer album={library.focusedAlbum} onclose={() => library.closeFocus()} />
    </div>
  {/if}

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
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transition: opacity 0.1s ease-out, visibility 0s linear 0.1s;
  }

  .view-layer.visible {
    opacity: 1;
    visibility: visible;
    transition: opacity 0.1s ease-out, visibility 0s linear 0s;
  }

  .view-layer.instant {
    transition: none !important;
  }

  .view-layer.active {
    pointer-events: auto;
  }

  .view-layer.home {
    z-index: 1;
    background-color: var(--background-main);
  }

  .view-layer.queue {
    z-index: 2;
    background-color: var(--background-main);
  }

  .view-layer.shelves {
    z-index: 3;
    background-color: var(--background-main);
  }

  .modal-layer {
    position: absolute;
    inset: 0;
    z-index: 150;
  }
</style>
