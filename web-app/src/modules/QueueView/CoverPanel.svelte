<script>
  import ModalDrawerCover from "../HomeView/ModalDrawer/ModalDrawerCover.svelte";

  let { coverUrl = "", onclick, width = $bindable(0) } = $props();
</script>

<div 
  class="cover-panel" 
  class:clickable={!!coverUrl}
  bind:clientWidth={width}
  {onclick}
  role="button"
  tabindex="0"
  onkeydown={(e) => { if(e.key === 'Enter') onclick?.(); }}
>
  <div class="cover-absolute-wrapper">
    {#if coverUrl && width > 0}
      <ModalDrawerCover 
        src={coverUrl} 
        width={width} 
        height={width} 
      />
    {:else if !coverUrl}
      <div class="empty-cover">
        <span>NO SIGNAL</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .cover-panel {
    flex: 0 1 auto;
    height: 100%;
    aspect-ratio: 1 / 1;
    max-width: 60vw;
    align-self: center;
    background-color: transparent;
    position: relative;
    cursor: default;
    outline: none;
    border: none;
  }

  .cover-panel.clickable {
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
</style>
