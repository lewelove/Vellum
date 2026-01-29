<script>
  import { theme } from "../../theme.svelte.js";

  let { album, active, onclick, scrollY = 0, rowY = 0 } = $props();

  let coverUrl = $derived(album.cover_hash 
    ? `/api/covers/${album.cover_hash}.png` 
    : "");

  const coverSize = $derived(theme.albumGrid["cover-size"]);
  const gapY = $derived(theme.albumGrid["gap-y"]);
  const textGap = $derived(theme.albumGrid["text-gap-main"]);

  // Use absolute top of viewport (0) as the occlusion boundary
  const occlusionBoundary = 0;

  let absoluteY = $derived(rowY - scrollY);
  let metadataTop = $derived(absoluteY + gapY + coverSize + textGap);
  
  let opacity = $derived.by(() => {
    const fadeDistance = 40;
    const diff = metadataTop - occlusionBoundary;
    return Math.max(0, Math.min(1, diff / fadeDistance));
  });

  let clipAmount = $derived.by(() => {
    const diff = occlusionBoundary - metadataTop;
    return Math.max(0, diff);
  });
</script>

<div class="album-unit">
  <button 
    class="album-cover" 
    class:active
    style="
      {coverUrl ? `background-image: url('${coverUrl}');` : ''}
      z-index: 10;
    "
    {onclick}
    aria-label="Select album {album.title}"
  ></button>
  
  <div 
    class="album-info" 
    style="
      opacity: {opacity};
      clip-path: inset({clipAmount}px 0 0 0);
      z-index: 1;
    "
  >
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
    position: relative;
    -webkit-font-smoothing: subpixel-antialiased;
    text-rendering: optimizeLegibility;
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
    background-color: #323232;
    background-size: cover;
    background-position: center;
    border-radius: 0px;
    box-shadow: var(--album-cover-shadow);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
    pointer-events: auto;
  }

  .album-info {
    display: flex;
    flex-direction: column;
    text-align: left;
    position: relative;
    will-change: opacity, clip-path;
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
