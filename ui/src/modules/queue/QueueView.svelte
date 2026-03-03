<script>
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
</script>

<div class="queue-view-container">
  
  <div class="view-content-wrapper">
    <QueueHud>
      <div class="modal-queue-chassis">
        
        <div 
          class="column-left" 
          bind:clientWidth={leftColumnWidth}
          bind:clientHeight={leftColumnHeight}
        >
          {#if coverSize > 0}
            <div class="cover-wrapper" style="width: {coverSize}px; height: {coverSize}px;">
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
    position: absolute;
    inset: 24px 32px 12px 32px;
    background-color: #242424;
    border-radius: 12px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.05);
    box-shadow: var(--album-cover-shadow);
    pointer-events: auto;
    display: grid;
    grid-template-columns: 60% 40%;
    grid-template-rows: 100%;
  }

  /* --- Left Column --- */
  .column-left {
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: #1f1f1f;
    border-right: 1px solid rgba(255, 255, 255, 0.05);
    min-width: 0;
    min-height: 0;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
  }

  .cover-wrapper {
    position: relative;
    flex-shrink: 0;
  }

  .empty-cover {
    width: 100%;
    height: 100%;
    background-color: #111;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .empty-cover span {
    font-family: var(--font-mono);
    color: #333;
    font-size: 12px;
    letter-spacing: 2px;
  }

  /* --- Right Column --- */
  .column-right {
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
</style>
