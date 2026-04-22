<script>
  import { onMount, onDestroy } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { nav } from "../../navigation.svelte.js";
  import { fade } from "svelte/transition";
  
  import TracklistPanel from "./TracklistPanel.svelte";
  import Sidebar from "./Sidebar.svelte";
  import LyricsPanel from "./LyricsPanel.svelte";
  import ClearCover from "../ClearCover.svelte";
  import BackgroundShader from "./BackgroundShader.svelte";
  import NavBar from "../NavigationBar/NavBar.svelte";
  import CoverPanel from "./CoverPanel.svelte";

  let activeId = $derived(player.currentAlbumId);
  let activeAlbum = $derived(activeId ? library.dict[activeId] : null);
  let coverHash = $derived(activeAlbum?.cover_hash || "");
  
  let fullAlbum = $derived(activeId ? library.fullAlbumCache[activeId] : null);

  let palette = $derived(fullAlbum?.album?.tags?.COVER_PALETTE || activeAlbum?.tags?.COVER_PALETTE || []);
  let hasLyrics = $derived(fullAlbum?.tracks?.some(t => !!t.info?.lyrics_path || t.tags?.INSTRUMENTAL === true) ?? false);

  let isViewVisible = $derived(nav.activeTab === 'queue');
  let isPlaying = $derived(player.state === "play");

  let showLyricsPanel = $derived(library.queuePanels.lyrics && hasLyrics);
  let showTracksPanel = $derived(library.queuePanels.tracks);

  let moduleWidth = $state(0);

  let isExpanded = $state(false);
  let windowWidth = $state(0);
  let windowHeight = $state(0);

  let expandedSize = $derived.by(() => {
    if (windowWidth <= 0 || windowHeight <= 0) return 0;
    return Math.min(windowWidth, windowHeight) - 48; 
  });

  $effect(() => {
    if (activeId) {
      library.ensureFullAlbum(activeId);
    }
  });

  function toggleExpand() {
    if (coverHash) {
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

<div class="queue-view-container" class:shader-off={!library.isShaderActive}>
  <BackgroundShader colors={palette} coverSize={moduleWidth} visible={isViewVisible} {isPlaying} />

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
          <ClearCover 
            hash={coverHash} 
            width={expandedSize} 
            height={expandedSize} 
          />
        </div>
      </div>
    </div>
  {/if}
  
  <div class="view-content-wrapper">
    <div class="queue-modules">
      
      {#if showLyricsPanel}
        <div class="side-column">
          <div class="module-panel lyrics-panel v-glass">
            <div class="panel-inner">
              <LyricsPanel />
            </div>
          </div>
        </div>
      {/if}

      <CoverPanel 
        {coverHash} 
        bind:width={moduleWidth} 
        onclick={toggleExpand} 
      />

      {#if showTracksPanel}
        <div class="side-column">
          <div class="module-panel tracks-panel v-glass">
            <div class="panel-inner">
              <TracklistPanel />
            </div>
          </div>
        </div>
      {/if}

    </div>
  </div>

  <Sidebar {hasLyrics} />
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
    overflow: hidden;
  }

  .queue-modules {
    width: 100%;
    flex: 1;
    min-height: 0;
    min-width: 0;
    display: flex;
    flex-direction: row;
    justify-content: center;
    align-items: stretch;
  }

  .side-column {
    display: flex;
    max-width: 500px;
    flex-direction: column;
    flex: 1;
    gap: 16px;
    height: 100%;
    min-width: 0;
    justify-content: flex-start;
  }

  .module-panel {
    min-width: 250px;
    border-radius: 16px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tracks-panel {
    flex: 1;
    min-height: 0;
    border-radius: 0 20px 20px 0;
    clip-path: inset(-100px -100px -100px 0px);
  }

  .lyrics-panel {
    flex: 1;
    min-height: 0;
    border-radius: 20px 0 0 20px;
    clip-path: inset(-100px 0px -100px -100px);
  }

  .panel-inner {
    flex: 1;
    padding: 24px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .expanded-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background-color: rgba(0, 0, 0, 0.2);
    backdrop-filter: blur(16px);
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
