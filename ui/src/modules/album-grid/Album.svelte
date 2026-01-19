<script>
  let { album, active, onclick } = $props();

  // URL encode the ID to be safe for the path
  let coverUrl = $derived(`/api/assets/${encodeURIComponent(album.id)}/cover`);
</script>

<div class="album-unit">
  <button 
    class="album-cover" 
    class:active
    style="background-color: {album.color}; background-image: url('{coverUrl}');"
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
    width: var(--cover-size);
    padding-top: var(--gap-y);
    /* No Stacking Context here */
    position: relative;
  }

  .album-cover {
    border: none;
    padding: 0;
    cursor: pointer;
    display: block;
    outline: none !important;
    width: var(--cover-size);
    height: var(--cover-size);
    margin-bottom: var(--text-gap-main);
    /* Slide OVER the crease */
    position: relative;
    z-index: 2;
    background-size: cover;
    background-position: center;
  }

  .album-info {
    display: flex;
    flex-direction: column;
    text-align: left;
    /* Slide UNDER the crease (implicit z-index: 0) */
    position: relative;
    z-index: 0;
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
    margin-bottom: var(--text-gap-lesser);
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
  }
</style>
