<script>
  import { onMount, onDestroy } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { nav } from "../../navigation.svelte.js";
  import { fade } from "svelte/transition";
  
  import QueueTracks from "./QueueTracks.svelte";
  import QueueBar from "./QueueBar.svelte";
  import Lyrics from "./Lyrics.svelte";
  import ModalDrawerCover from "../album-grid/ModalDrawerCover.svelte";
  import QueueBackgroundShader from "./QueueBackgroundShader.svelte";
  import NavBar from "../navigation/NavBar.svelte";

  let activeId = $derived(player.currentAlbumId);
  let activeAlbum = $derived(activeId ? library.albumCache.get(activeId) : null);
  let coverUrl = $derived(activeId ? library.getAlbumCoverUrl(activeId) : "");
  
  let palette = $derived(activeAlbum?.tags?.COVER_PALETTE || []);

  let isViewVisible = $derived(nav.activeTab === 'queue');
  let isPlaying = $derived(player.state === "play");

  let panels = $state({
    lyrics: false,
    tracks: true
  });

  function togglePanel(key) {
    panels[key] = !panels[key];
  }

  // Padding configuration
  const coverPadding = 24;
  let moduleWidth = $state(0);
  
  // Explicit calculation: subtract padding from both sides
  let coverSize = $derived(Math.max(0, moduleWidth - (coverPadding * 2)));

  let isExpanded = $state(false);
  let windowWidth = $state(0);
  let windowHeight = $state(0);

  let expandedSize = $derived.by(() => {
    if (windowWidth <= 0 || windowHeight <= 0) return 0;
    return Math.min(windowWidth, windowHeight) - 48; 
  });

  function toggleExpand() {
    if (coverUrl) {
      isExpanded = !isExpanded;
    }
  }

  function handleKeydown(e) {
    if (isExpanded && e.key === "Escape") {
      isExpanded = false;
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
</script>

<svelte:window bind:innerWidth={windowWidth} bind:innerHeight={windowHeight} />

<div class="queue-view-container">
  <QueueBackgroundShader colors={palette} coverSize={coverSize} visible={isViewVisible} {isPlaying} />

  <NavBar variant="glass" />

  {#if isExpanded}
    <div 
      class="expanded-backdrop" 
      onclick={toggleExpand}
      role="button"
      tabindex="0"
      onkeydown={(e) => { if(e.key === 'Enter') toggleExpand(); }}
      transition:fade={{ duration: 300 }}
    >
      <div 
        class="expanded-content" 
        style="width: {expandedSize}px; height: {expandedSize}px;"
        onclick={(e) => e.stopPropagation()} 
        role="presentation"
      >
        <div in:fade={{ duration: 100 }}>
          <ModalDrawerCover 
            src={coverUrl} 
            width={expandedSize} 
            height={expandedSize} 
          />
        </div>
      </div>
    </div>
  {/if}
  
  <div class="view-content-wrapper">
    
    <div class="queue-modules">
      {#if panels.lyrics}
        <div class="module-panel">
          <div class="panel-inner">
            <Lyrics />
          </div>
        </div>
      {/if}

      <div 
        class="module-panel module-cover" 
        class:clickable={!!coverUrl}
        bind:clientWidth={moduleWidth}
        onclick={toggleExpand}
        role="button"
        tabindex="0"
        onkeydown={(e) => { if(e.key === 'Enter') toggleExpand(); }}
      >
        <div class="panel-inner cover-inner" style="padding: {coverPadding}px;">
          <div class="cover-absolute-wrapper" style="inset: {coverPadding}px;">
            {#if coverUrl}
              <ModalDrawerCover 
                src={coverUrl} 
                width={coverSize} 
                height={coverSize} 
              />
            {:else}
              <div class="empty-cover">
                <span>NO SIGNAL</span>
              </div>
            {/if}
          </div>
        </div>
      </div>

      {#if panels.tracks}
        <div class="module-panel greedy">
          <div class="panel-inner">
            <QueueTracks />
          </div>
        </div>
      {/if}
    </div>

  </div>

  <QueueBar {panels} onToggle={togglePanel} />
</div>

<style>
  .queue-view-container {
    width: 100%;
    height: 100%;
    background-color: transparent;
    position: relative;
    overflow: hidden;
    display: flex;
    flex-direction: row;
  }

  .view-content-wrapper {
    flex: 1;
    position: relative;
    height: 100%;
    min-width: 0;
    padding: 24px; 
    box-sizing: border-box;
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 32px;
    overflow: hidden;
  }

  .queue-modules {
    width: 100%;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: row;
    gap: 16px;
    justify-content: center;
    align-items: flex-start;
  }

  .module-panel {
    flex: 0 1 auto;
    min-width: 240px;
    height: auto;
    max-height: 100%;
    background-color: #24242480;
    backdrop-filter: blur(8px);
    border-radius: 12px;
    box-shadow: 0 0 16px rgba(0, 0, 0, 0.1), 0 0 16px rgba(0, 0, 0, 0.2), 0 0 10px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .module-panel.greedy {
    flex: 1 1 0;
  }

  .panel-inner {
    flex: 1;
    padding: 24px 24px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .module-cover {
    flex: 0 1 auto;
    height: 100%;
    width: auto;
    aspect-ratio: 1 / 1;
    cursor: default;
    outline: none;
    background-color: #24242442;
    backdrop-filter: blur(0px);
    /* border-radius: 0px !important; */
  }

  .module-cover.clickable {
    cursor: pointer;
  }

  .cover-inner {
    position: relative;
    width: 100%;
    height: 100%;
    /* Override global padding to use local variable */
    padding: 0 !important; 
  }
  
  .cover-absolute-wrapper {
    position: absolute;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-cover {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(255, 255, 255, 0.05);
    box-sizing: border-box;
  }

  .empty-cover span {
    font-family: var(--font-mono);
    color: #444;
    font-size: 12px;
    letter-spacing: 2px;
  }

  .expanded-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background-color: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(25px);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .expanded-content {
    position: relative;
    z-index: 10000;
    pointer-events: none;
  }
</style>
