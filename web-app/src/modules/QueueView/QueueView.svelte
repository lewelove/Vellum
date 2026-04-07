<script>
  import { onMount, onDestroy } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { nav } from "../../navigation.svelte.js";
  import { fade } from "svelte/transition";
  
  import TrackList from "./TrackList.svelte";
  import Sidebar from "./Sidebar.svelte";
  import LyricsPanel from "./LyricsPanel.svelte";
  import ClearCover from "../ClearCover.svelte";
  import BackgroundShader from "./BackgroundShader.svelte";
  import NavBar from "../NavigationBar/NavBar.svelte";
  import ControlPanel from "./ControlPanel.svelte";
  import CoverPanel from "./CoverPanel.svelte";

  let activeId = $derived(player.currentAlbumId);
  let activeAlbum = $derived(activeId ? library.albumCache.get(activeId) : null);
  let coverUrl = $derived(activeId ? library.getAlbumCoverUrl(activeId) : "");
  
  let palette = $derived(activeAlbum?.tags?.COVER_PALETTE || []);
  let hasLyrics = $derived(activeAlbum?.tracks?.some(t => !!t.lyrics_path) ?? false);

  let isViewVisible = $derived(nav.activeTab === 'queue');
  let isPlaying = $derived(player.state === "play");

  let panels = $state({
    lyrics: false,
    tracks: true
  });

  let noSidePanels = $derived(!panels.lyrics && !panels.tracks);

  $effect(() => {
    if (!hasLyrics && panels.lyrics) {
      panels.lyrics = false;
    }
  });

  function togglePanel(key) {
    panels[key] = !panels[key];
  }

  let moduleWidth = $state(0);

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
        <div class="side-column">
          <div class="module-panel lyrics-panel v-glass">
            <div class="panel-inner">
              <LyricsPanel />
            </div>
          </div>
        </div>
      {/if}

      <CoverPanel 
        {coverUrl} 
        bind:width={moduleWidth} 
        onclick={toggleExpand} 
      />

      {#if panels.tracks}
        <div class="side-column">
          <div class="module-panel tracks-panel v-glass">
            <div class="panel-inner">
              <TrackList />
            </div>
          </div>
        </div>
      {/if}

    </div>
    
    <div 
      class="control-wrapper" 
      class:constrained={noSidePanels && moduleWidth > 0} 
      style="--cover-width: {moduleWidth}px;"
    >
      <ControlPanel />
    </div>
  </div>

  <Sidebar {panels} {hasLyrics} onToggle={togglePanel} />
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
    padding: 20px 32px 16px; 
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
    gap: 16px;
    justify-content: center;
    align-items: stretch;
  }

  .side-column {
    display: flex;
    flex-direction: column;
    flex: 1;
    gap: 16px;
    height: 100%;
    min-width: 0;
    justify-content: flex-start;
  }

  .module-panel {
    min-width: 240px;
    border-radius: 16px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tracks-panel {
    flex: 1;
    min-height: 0;
  }

  .lyrics-panel {
    flex: 1;
    min-height: 0;
  }

  .panel-inner {
    flex: 1;
    padding: 24px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .control-wrapper {
    width: 100%;
    margin: 0 auto;
    display: flex;
    flex-shrink: 0;
    transition: max-width 0.2s cubic-bezier(0.2, 0, 0, 1);
  }

  .control-wrapper.constrained {
    max-width: var(--cover-width);
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
