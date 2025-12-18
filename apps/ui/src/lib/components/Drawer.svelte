<script>
  let { activeAlbum, width } = $props();
</script>

<div class="drawer" style="width: {width}px;">
  <div class="drawer-content">
    <h2>{activeAlbum.title}</h2>
    <h3>{activeAlbum.artist}</h3>
    <ul>
      {#each activeAlbum.tracks as track}
        <li>{track}</li>
      {/each}
    </ul>
  </div>
</div>
