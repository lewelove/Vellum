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

<style>
  .drawer {
    background-color: var(--drawer-bg);
    /* 
       REMOVED: border-left: 4px solid var(--brand-red); 
       The drawer now integrates seamlessly into the background.
    */
    box-sizing: border-box;
    animation: slideDown 0.2s ease-out;
    margin: 0 auto 20px auto; 
  }

  .drawer-content {
    /* 
       Visual Note: You may want to adjust this padding if the 
       lack of a border makes the text feel too "un-anchored" 
       on the left side. 
    */
    padding: 20px;
  }

  h2 { margin: 0 0 5px 0; color: var(--highlight); font-size: 24px; font-weight: normal; }
  h3 { margin: 0 0 20px 0; color: var(--text-accent); font-size: 16px; font-weight: normal; }
  ul { list-style: none; padding: 0; margin: 0; }
  li { padding: 8px 0; border-bottom: 1px solid #333; font-size: 14px; color: var(--text-main); }
  li:last-child { border-bottom: none; }

  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
