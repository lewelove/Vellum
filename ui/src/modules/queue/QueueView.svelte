<script>
  import { onMount, onDestroy } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { fade } from "svelte/transition";
  
  import QueueTracks from "./QueueTracks.svelte";
  import QueueHud from "./QueueHud.svelte";
  import QueueBar from "./QueueBar.svelte";
  import Lyrics from "./Lyrics.svelte";
  import ModalDrawerCover from "../album-grid/ModalDrawerCover.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(activeId ? library.getAlbumCoverUrl(activeId) : "");

  let activeView = $state("tracks");

  let leftColumnWidth = $state(0);
  let leftColumnHeight = $state(0);
  
  let coverSize = $derived.by(() => {
    if (leftColumnWidth <= 0 || leftColumnHeight <= 0) return 0;
    const minDim = Math.min(leftColumnWidth, leftColumnHeight);
    return minDim - 64; 
  });

  let isExpanded = $state(false);
  let windowWidth = $state(0);
  let windowHeight = $state(0);

  let expandedSize = $derived.by(() => {
    if (windowWidth <= 0 || windowHeight <= 0) return 0;
    return Math.min(windowWidth, windowHeight) - 48; // 24px * 2
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

  {#if isExpanded}
    <div 
      class="expanded-backdrop" 
      onclick={toggleExpand}
      role="button"
      tabindex="0"
      onkeydown={(e) => { if(e.key === 'Enter') toggleExpand(); }}
    >
      <div 
        class="expanded-content" 
        style="width: {expandedSize}px; height: {expandedSize}px;"
        onclick={(e) => e.stopPropagation()} 
        role="presentation"
      >
        <ModalDrawerCover 
          src={coverUrl} 
          width={expandedSize} 
          height={expandedSize} 
        />
      </div>
    </div>
  {/if}
  
  <div class="view-content-wrapper">
    <QueueHud>
      <div class="modal-queue-chassis">
        
        <div 
          class="column-left" 
          bind:clientWidth={leftColumnWidth}
          bind:clientHeight={leftColumnHeight}
        >
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

        <div class="column-right">
          <div class="scroll-area">
            <div class="scroll-fade-overlay-top"></div>
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
            <div class="scroll-fade-overlay-bottom"></div>
          </div>
        </div>

      </div>
    </QueueHud>
  </div>

  <QueueBar {activeView} onViewChange={(v) => activeView = v} />

</div>

<style>
  .queue-view-container {
    width: 100%;
    height: 100%;
    background-color: var(--background-main);
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
  }

  .modal-queue-chassis {
    width: 100%;
    height: 100%;
    background-color: #242424;
    border-radius: 16px;
    overflow: hidden;
    box-shadow: var(--modal-shadow);
    pointer-events: auto;
    display: flex;
    flex-direction: row;
  }

  .column-left {
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: #1f1f1f;
    min-width: 0;
    min-height: 0;
    height: 100%;
    aspect-ratio: 1 / 1;
    flex-shrink: 0;
    max-width: 60%;
    box-sizing: border-box;
  }

  .cover-wrapper {
    position: relative;
    flex-shrink: 0;
    transition: transform 0.1s ease;
    outline: none;
    -webkit-tap-highlight-color: transparent;
  }

  .cover-wrapper.clickable {
    cursor: pointer;
  }

  .empty-cover {
    width: 100%;
    height: 100%;
    background-color: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .empty-cover span {
    font-family: var(--font-mono);
    color: #1F1F1F;
    font-size: 12px;
    letter-spacing: 2px;
  }

  .column-right {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 32px;
    min-width: 0;
    min-height: 0;
    height: 100%;
    box-sizing: border-box;
    background-color: #242424;
  }

  .scroll-area {
    position: relative;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
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

  .scroll-fade-overlay-top {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 12px;
    background: linear-gradient(to bottom, #242424 0%, transparent 100%);
    z-index: 10;
    pointer-events: none;
  }

  .scroll-fade-overlay-bottom {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 12px;
    background: linear-gradient(to top, #242424 0%, transparent 100%);
    z-index: 10;
    pointer-events: none;
  }

  .expanded-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background-color: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    outline: none;
  }

  .expanded-content {
    position: relative;
    /* box-shadow: 0 0 32px rgba(0,0,0,0.5); */
    background-color: transparent;
  }
</style>
