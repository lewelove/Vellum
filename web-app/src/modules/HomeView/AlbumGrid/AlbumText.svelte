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
      const ctx = canvas.getContext('2d', { alpha: false });
      const dpr = window.devicePixelRatio || 1;
      
      canvas.width = coverSize * dpr;
      canvas.height = (textBlockHeight + 2) * dpr;
      
      ctx.scale(dpr, dpr);
      ctx.translate(0, 1);
      
      const bgHex = "#333333";
      ctx.fillStyle = bgHex;
      ctx.fillRect(0, -1, coverSize, textBlockHeight + 2);
      
      ctx.drawImage(textBitmap, 0, -1, coverSize, textBlockHeight + 2);
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
      height: calc(100% + 2px);
      position: absolute;
      top: -1px;
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
