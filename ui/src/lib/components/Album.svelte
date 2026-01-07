<script>
  let { album, size, active, textHeight, onclick } = $props();
  // Fixed internal layout to ensure the unit height matches layout.rowHeight
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
  }

  .album-info {
    display: flex;
    flex-direction: column;
    text-align: left;
    overflow: hidden;
  }

  .album-title {
    display: block;
    font-size: 14px;
    font-weight: 700;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .album-artist {
    display: block;
    font-size: 12px;
    color: var(--text-accent); 
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
