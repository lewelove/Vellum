<script>
  let { hash, width, height } = $props();

  let dpr = $derived(window.devicePixelRatio || 1);
  let targetWidth = $derived(Math.round(width * dpr));

  let srcUrl = $derived(hash && targetWidth > 0 ? `/api/resize/${targetWidth}px/${hash}?v=${hash}` : "");

  let isLoaded = $state(false);

  $effect(() => {
    let _ = srcUrl;
    isLoaded = false;
  });
</script>

<div class="clear-cover-wrapper" style="width: {width}px; height: {height}px;">
  {#if srcUrl}
    <div class="cover-block" class:visible={isLoaded}>
      <img
        src={srcUrl}
        class="cover-image"
        alt=""
        draggable="false"
        onload={() => isLoaded = true}
      />
    </div>
  {:else}
    <div class="empty-cover">
      <span>NO SIGNAL</span>
    </div>
  {/if}
</div>

<style>
  .clear-cover-wrapper {
    position: relative;
    overflow: visible;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cover-block {
    position: absolute;
    inset: 0;
    opacity: 0;
    transition: opacity 0.2s ease;
    will-change: opacity 0.2s ease;
  }

  .cover-block.visible {
    opacity: 1;
  }

  .cover-image {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    box-shadow: var(--album-cover-shadow);
  }

  .empty-cover {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
  }

  .empty-cover span {
    font-family: var(--font-mono);
    color: #444;
    font-size: 12px;
    letter-spacing: 2px;
  }
</style>
