<script>
  let { album, size, active, onclick } = $props();
</script>

<div class="album-unit" style="width: {size}px;">
  <button 
    class="album-cover" 
    class:active
    style="width: {size}px; height: {size}px; background-color: {album.color};"
    {onclick}
  ></button>
  <div class="album-info">
    <span class="album-title">{album.title}</span>
    <span class="album-artist">{album.artist}</span>
  </div>
</div>

<style>
  .album-unit {
    display: flex;
    flex-direction: column;
    gap: 8px; 
    flex-shrink: 0; 
  }

  .album-cover {
    border: none;
    padding: 0;
    cursor: pointer;
    transition: filter 0.2s;
    display: block;

    /* 
       Ensure component-level isolation from browser focus chrome. 
       !important is used to override any agent stylesheets.
    */
    outline: none !important;
    box-shadow: none !important;
    -webkit-tap-highlight-color: transparent;
  }

  .album-cover:hover {
    filter: brightness(1.04);
  }

  /* Redundant safety for keyboard interaction states */
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
  }

  .album-title {
    display: block;
    font-weight: bold;
    font-size: 14px;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 1px;
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
