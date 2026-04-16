<script>
  import ClearCover from "../ClearCover.svelte";

  let { coverUrl = "", onclick, width = $bindable(0) } = $props();
</script>

<div class="cover-wrapper v-glass">
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
      {#if width > 0}
        <ClearCover 
          src={coverUrl} 
          width={width} 
          height={width} 
        />
      {/if}
    </div>
  </div>
</div>

<style>
  .cover-wrapper {
    flex: 0 1 auto;
    height: 100%;
    max-height: 100%;
    max-width: 60%;
    aspect-ratio: 1 / 1;
    align-self: center;
    min-width: 0;
    min-height: 0;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .cover-panel {
    width: 100%;
    height: 100%;
    position: relative;
    cursor: default;
    outline: none;
    border: none;
    box-sizing: border-box;
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
</style>
