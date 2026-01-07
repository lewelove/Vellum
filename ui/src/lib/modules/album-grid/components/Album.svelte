<script>
  let { album, active, onclick } = $props();
</script>

<div class="album-unit">
  <button 
    class="album-cover" 
    class:active
    style="background-color: {album.color};"
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
    flex-shrink: 0; 
    width: var(--grid-cover-size);
    /* Row starts with the top gap */
    padding-top: var(--grid-gap-main);
  }

  .album-cover {
    border: none;
    padding: 0;
    cursor: pointer;
    display: block;
    outline: none !important;
    width: var(--grid-cover-size);
    height: var(--grid-cover-size);
    margin-bottom: var(--grid-text-gap-main);
  }

  .album-info {
    display: flex;
    flex-direction: column;
    text-align: left;
    overflow: hidden;
  }

  .album-title {
    display: block;
    font-size: var(--font-size-title);
    font-weight: var(--font-weight-title);
    line-height: var(--font-size-title);
    height: var(--font-size-title);
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: var(--grid-text-gap-lesser);
  }

  .album-artist {
    display: block;
    font-size: var(--font-size-artist);
    font-weight: var(--font-weight-artist);
    line-height: var(--font-size-artist);
    height: var(--font-size-artist);
    color: var(--text-muted); 
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    /* No margin or padding here: the row ends exactly at this line */
  }
</style>
