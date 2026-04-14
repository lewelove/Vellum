<script>
  import { theme } from "../../../theme.svelte.js";
  import { library } from "../../../library.svelte.js";

  let { album, scrollY = 0, rowY = 0 } = $props();

  const coverSize = $derived(theme.albumGrid["cover-size"]);
  const gapY = $derived(theme.albumGrid["gap-y"]);
  const textGap = $derived(theme.albumGrid["text-gap-main"]);

  const lhTitle = $derived(theme.albumGrid["font-line-height-title"]);
  const gapLesser = $derived(theme.albumGrid["text-gap-lesser"]);
  const lhArtist = $derived(theme.albumGrid["font-line-height-artist"]);
  const textBlockHeight = $derived(lhTitle + gapLesser + lhArtist);

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

  let canvas;
  let textBitmap = $derived(library.pinnedTextTextures.get(album.id));

  $effect(() => {
    if (canvas && textBitmap) {
      const ctx = canvas.getContext('2d', { alpha: true });
      const dpr = window.devicePixelRatio || 1;
      
      const targetWidth = coverSize * dpr;
      const targetHeight = textBlockHeight * dpr;

      if (canvas.width !== targetWidth || canvas.height !== targetHeight) {
          canvas.width = targetWidth;
          canvas.height = targetHeight;
      }
      
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(textBitmap, 0, 0, canvas.width, canvas.height);
    }
  });
</script>

<div 
  class="album-info" 
  style="
    opacity: {opacity};
    clip-path: inset({clipAmount}px 0 0 0);
    z-index: 1;
    height: {textBlockHeight}px;
  "
>
  <canvas 
    bind:this={canvas}
    style="
      width: {coverSize}px;
      height: {textBlockHeight}px;
      position: absolute;
      top: 0;
      left: 0;
      display: block;
      image-rendering: -webkit-optimize-contrast;
    "
  ></canvas>
</div>

<style>
  .album-info {
    display: block;
    position: relative;
    will-change: opacity, clip-path;
  }
</style>
