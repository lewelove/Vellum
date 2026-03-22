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
  
  let palette = $derived(activeAlbum?.tags?.COVER_PALETTE || []);

  let isViewVisible = $derived(nav.activeTab === 'queue');

  let activeView = $state("tracks");

  let containerWidth = $state(0);
  let containerHeight = $state(0);
  
  const FOOTER_HEIGHT = 64;
  const PADDING = 64;

  let mainHeight = $derived(Math.max(0, containerHeight - FOOTER_HEIGHT));
  
  let leftPanelWidth = $derived.by(() => {
    if (containerHeight <= 0 || containerWidth <= 0) return 0;
    const maxWidth = containerWidth * 0.6;
    return Math.min(mainHeight, maxWidth);
  });

  let coverSize = $derived(Math.max(0, leftPanelWidth - PADDING));

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
    <div 
      class="queue-layout"
      bind:clientWidth={containerWidth}
      bind:clientHeight={containerHeight}
    >
      <div class="layout-main" style="height: {mainHeight}px;">
        <div class="column-left" style="width: {leftPanelWidth}px;">
          <div class="left-main-area">
            {#if coverSize > 0}
              <div 
                class="cover-wrapper" 
                class:clickable={!!coverUrl}
                style="width: {coverSize}px; height: {coverSize}px;"
                onclick={toggleExpand}
                role="button"
                tabindex="0"
                onkeydown={(e) => { if(e.key === 'Enter') toggleExpand(); }}
              >
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
            {/if}
          </div>
        </div>

        <div class="column-right">
          <div class="scroll-area">
            <div class="scroll-content">
              {#if activeView === 'tracks'}
                <div class="tracks-wrapper" in:fade={{ duration: 150 }}>
                  <QueueTracks />
                </div>
              {:else if activeView === 'lyrics'}
                <div class="lyrics-wrapper" in:fade={{ duration: 150 }}>
                  <Lyrics />
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>

      <div class="layout-footer">
        <ProgressBar />
      </div>
    </div>
  </div>

  <QueueBar {activeView} onViewChange={(v) => activeView = v} />
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
    padding: 32px; 
    box-sizing: border-box;
    z-index: 1;
  }

  .queue-layout {
    width: 100%;
    height: 100%;
    background: rgba(36, 36, 36, 1.66);
    -webkit-backdrop-filter: blur(50px);
    border-radius: 16px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 0 16px rgba(0, 0, 0, 0.1), 0 0 16px rgba(0, 0, 0, 0.2), 0 0 10px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
  }

  .layout-main {
    display: flex;
    flex-direction: row;
    min-height: 0;
    flex: 1;
  }

  .column-left {
    display: flex;
    flex-direction: column;
    background-color: rgba(0, 0, 0, 0.1);
    height: 100%;
    flex-shrink: 0;
    box-sizing: border-box;
    overflow: hidden;
    border-right: 1px solid rgba(255, 255, 255, 0.03);
  }

  .left-main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    padding: 32px;
    box-sizing: border-box;
  }

  .cover-wrapper {
    position: relative;
    flex-shrink: 0;
    outline: none;
  }

  .cover-wrapper.clickable {
    cursor: pointer;
  }

  .empty-cover {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .empty-cover span {
    font-family: var(--font-mono);
    color: #444;
    font-size: 12px;
    letter-spacing: 2px;
  }

  .column-right {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    box-sizing: border-box;
    background-color: transparent;
    overflow: hidden;
  }

  .scroll-area {
    position: relative;
    flex: 1;
    background-color: rgba(0, 0, 0, 0.1);
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding: 32px;
  }

  .scroll-content {
    flex: 1;
    position: relative;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .scroll-content::-webkit-scrollbar {
    width: 0px;
  }

  .tracks-wrapper, .lyrics-wrapper {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .layout-footer {
    flex-shrink: 0;
    width: 100%;
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
