<script>
  let { album, active, onclick } = $props();

  // New Content Addressable URL
  // If no hash exists (e.g. no cover), the server 404s and we show background color
  let coverUrl = $derived(album.cover_hash 
    ? `/api/covers/${album.cover_hash}.png` 
    : "");
</script>

<div class="album-unit">
  <button 
    class="album-cover" 
    class:active
    style="{coverUrl ? `background-image: url('${coverUrl}');` : ''}"
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
    position: relative;
    z-index: 2;
    background-color: #323232;
    background-size: cover;
    background-position: center;
    border-radius: 0px;
    box-shadow: var(--album-cover-shadow);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }

  .album-cover:hover {
    /* filter: brightness(1.1); */
  }

  .album-info {
    display: flex;
    flex-direction: column;
    text-align: left;
    position: relative;
    z-index: 0;
  }

  .album-title {
    display: block;
    font-size: var(--font-size-title);
    line-height: var(--font-line-height-title);
    height: var(--font-line-height-title);
    font-weight: var(--font-weight-title);
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: var(--text-gap-lesser);
  }

  .album-artist {
    display: block;
    font-size: var(--font-size-artist);
    line-height: var(--font-line-height-artist);
    height: var(--font-line-height-artist);
    font-weight: var(--font-weight-artist);
    color: var(--text-muted); 
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
