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
  import ProgressBar from "./ProgressBar.svelte";
  import QueueBackgroundShader from "./QueueBackgroundShader.svelte";

  let activeId = $derived(player.currentAlbumId);
  let activeAlbum = $derived(activeId ? library.albumCache.get(activeId) : null);
  let coverUrl = $derived(activeId ? library.getAlbumCoverUrl(activeId) : "");
  
  let palette = $derived(activeAlbum?.tags?.COVER_PALETTE ||[]);

  let isViewVisible = $derived(nav.activeTab === 'queue');

  let panels = $state({
    lyrics: false,
    tracks: true
  });

  function togglePanel(key) {
    panels[key] = !panels[key];
  }

  let coverSize = $state(0);

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
  <QueueBackgroundShader colors={palette} coverSize={coverSize} visible={isViewVisible} />

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
        <div class="module-panel" in:fade={{ duration: 150 }} out:fade={{ duration: 150 }}>
          <div class="panel-inner">
            <Lyrics />
          </div>
        </div>
      {/if}

      <div 
        class="module-cover" 
        class:clickable={!!coverUrl}
        bind:clientWidth={coverSize}
        onclick={toggleExpand}
        role="button"
        tabindex="0"
        onkeydown={(e) => { if(e.key === 'Enter') toggleExpand(); }}
      >
        <div class="cover-absolute-wrapper">
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

      {#if panels.tracks}
        <div class="module-panel" in:fade={{ duration: 150 }} out:fade={{ duration: 150 }}>
          <div class="panel-inner">
            <QueueTracks />
          </div>
        </div>
      {/if}
    </div>

    <div class="queue-bottom-bar">
      <ProgressBar />
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
    padding: 32px 32px 0 32px; 
    box-sizing: border-box;
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .queue-modules {
    width: 100%;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: row;
    gap: 32px;
    justify-content: center;
    align-items: center;
  }

  .module-panel {
    flex: 1 1 320px;
    /* min-width: 240px; */
    /* max-width: 800px; */
    height: 100%;
    background-color: rgba(36, 36, 36, 0.66);
    backdrop-filter: blur(20px);
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 0 16px rgba(0, 0, 0, 0.1), 0 0 16px rgba(0, 0, 0, 0.2), 0 0 10px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-inner {
    flex: 1;
    padding: 32px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .module-cover {
    flex: 0 1 auto;
    height: 100%;
    max-height: 100%;
    aspect-ratio: 1 / 1;
    display: flex;
    align-items: center;
    justify-content: center;
    outline: none;
    position: relative;
    min-width: 0;
    min-height: 0;
  }

  .module-cover.clickable {
    cursor: pointer;
  }
  
  .cover-absolute-wrapper {
    position: absolute;
    inset: 0;
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

  .queue-bottom-bar {
    width: 100%;
    height: 52px;
    flex-shrink: 0;
    background-color: rgba(36, 36, 36, 0.66);
    backdrop-filter: blur(30px);
    transform: translateZ(0);
    border-radius: 16px 16px 0 0;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-bottom: none;
    box-shadow: 0 -4px 16px rgba(0, 0, 0, 0.1), 0 -2px 10px rgba(0, 0, 0, 0.2);
    overflow: hidden;
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
