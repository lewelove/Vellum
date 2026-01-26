<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import QueueTracks from "./QueueTracks.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? `/api/assets/${encodeURIComponent(activeId)}/cover` : ""
  );

  let sidebarWidth = $state(320);
  let isResizing = $state(false);

  function startResizing(e) {
    isResizing = true;
    const startX = e.clientX;
    const startWidth = sidebarWidth;

    const onMouseMove = (moveEvent) => {
      const delta = startX - moveEvent.clientX;
      sidebarWidth = Math.max(200, Math.min(window.innerWidth - 100, startWidth + delta));
    };

    const onMouseUp = () => {
      isResizing = false;
      localStorage.setItem("eluxum-queue-width", sidebarWidth);
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

  onMount(() => {
    const saved = localStorage.getItem("eluxum-queue-width");
    if (saved) sidebarWidth = parseInt(saved);
  });
</script>

<div class="queue-view-container" class:resizing={isResizing}>
  <div class="cover-area">
    {#if coverUrl}
      <div class="fullscreen-cover">
        <img src={coverUrl} alt="Now Playing" class="now-playing-cover" />
      </div>
    {:else}
      <div class="empty-state">
        <span>Not Playing</span>
      </div>
    {/if}
  </div>

  <div class="resizer" onmousedown={startResizing}></div>

  <div class="tracks-area" style="width: {sidebarWidth}px">
    <QueueTracks />
  </div>
</div>

<style>
  .queue-view-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: row;
    background-color: var(--background-drawer);
    overflow: hidden;
  }

  .queue-view-container.resizing {
    cursor: col-resize;
    user-select: none;
  }

  .cover-area {
    flex: 1;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: #000;
    overflow: hidden;
  }

  .fullscreen-cover {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .now-playing-cover {
    height: 100%;
    width: 100%;
    object-fit: contain;
    display: block;
  }

  .resizer {
    width: 1px;
    height: 100%;
    background-color: var(--border-muted);
    cursor: col-resize;
    position: relative;
    z-index: 10;
  }

  .resizer::after {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: -4px;
    right: -4px;
  }

  .tracks-area {
    height: 100%;
    flex-shrink: 0;
    border-left: 1px solid rgba(0, 0, 0, 0.2);
  }

  .empty-state {
    color: var(--text-muted);
    font-size: 18px;
    letter-spacing: 0.3em;
    text-transform: uppercase;
    font-weight: 500;
  }
</style>
