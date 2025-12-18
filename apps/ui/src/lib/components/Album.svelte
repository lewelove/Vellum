<script>
  let { album, size, active, onclick } = $props();
</script>

<div class="album-unit" style="width: {size}px;">
  <button 
    class="album-cover" 
    class:active
    style="width: {size}px; height: {size}px; background-color: {album.color};"
    {onclick}
    aria-label="View {album.title}"
  ></button>
  <div class="album-info">
    <span class="album-title">{album.title}</span>
    <span class="album-artist">{album.artist}</span>
  </div>
</div>
