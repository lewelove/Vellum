<script>
  let { album, size, active, textHeight, onclick } = $props();
  
  // The internal gap between cover and text is 8px
  // We subtract it from the total textHeight allocation to keep the grid unit perfect
  let infoHeight = $derived(textHeight - 8);
</script>

<div class="album-unit" style="width: {size}px; gap: 8px;">
  <button 
    class="album-cover" 
    class:active
    style="width: {size}px; height: {size}px; background-color: {album.color};"
    {onclick}
  ></button>
  
  <div class="album-info" style="height: {infoHeight}px;">
    <span class="album-title">{album.title}</span>
    <span class="album-artist">{album.artist}</span>
  </div>
</div>

<style>
  .album-unit {
    display: flex;
    flex-direction: column;
    flex-shrink: 0; 
  }

  .album-cover {
    border: none;
    padding: 0;
    cursor: pointer;
    transition: filter 0.2s;
    display: block;
    outline: none !important;
    box-shadow: none !important;
    -webkit-tap-highlight-color: transparent;
  }

  .album-cover:focus, 
  .album-cover:active,
  .album-cover:focus-visible {
    outline: none !important;
    box-shadow: none !important;
  }

  .album-info {
    display: flex;
    flex-direction: column;
    text-align: left;
    overflow: hidden;
    /* Ensure text starts immediately after the 8px gap */
    justify-content: flex-start;
  }

  .album-title {
    display: block;
    font-size: 14px;
    font-weight: 700;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
    margin-top: 2px;
  }

  .album-artist {
    display: block;
    font-size: 12px;
    color: var(--text-accent); 
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }
</style>
