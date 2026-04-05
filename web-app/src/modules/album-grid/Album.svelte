<script>
  import { theme } from "../../theme.svelte.js";
  import { library } from "../../library.svelte.js";

  let { album, active, onclick, scrollY = 0, rowY = 0 } = $props();

  let originalUrl = $derived(library.getThumbnailUrl(album));
  let coverUrl = $derived(library.pinnedTextures.get(originalUrl) || originalUrl);

  const coverSize = $derived(theme.albumGrid["cover-size"]);
  const gapY = $derived(theme.albumGrid["gap-y"]);
  const textGap = $derived(theme.albumGrid["text-gap-main"]);

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
  
  const lhTitle = $derived(theme.albumGrid["font-line-height-title"]);
  const gapLesser = $derived(theme.albumGrid["text-gap-lesser"]);
  const lhArtist = $derived(theme.albumGrid["font-line-height-artist"]);
  const textBlockHeight = $derived(lhTitle + gapLesser + lhArtist);
  
  function fitText(ctx, text, maxWidth) {
    if (!text) return "";
    let ellipsis = "...";
    let width = ctx.measureText(text).width;
    if (width <= maxWidth) return text;
    
    let len = text.length;
    while (width > maxWidth && len > 0) {
      len--;
      width = ctx.measureText(text.substring(0, len) + ellipsis).width;
    }
    return text.substring(0, len) + ellipsis;
  }

  function applyEffects(ctx) {
    ctx.shadowColor = "rgba(0, 0, 0, 0.1)";
    ctx.shadowBlur = 4;
    ctx.shadowOffsetX = 0;
    ctx.shadowOffsetY = 0;
  }

  function renderText() {
    if (!canvas) return;
    
    const dpr = window.devicePixelRatio || 1;
    const w = coverSize;
    const h = textBlockHeight;

    if (w <= 0 || h <= 0) return;

    canvas.width = w * dpr;
    canvas.height = h * dpr;
    
    const ctx = canvas.getContext('2d', { alpha: false });
    ctx.scale(dpr, dpr);
    
    const bgHex = theme.palette[theme.colors["background-main"]] || "#323232";
    ctx.fillStyle = bgHex;
    ctx.fillRect(0, 0, w, h);
    
    const fontStack = "Inter, 'Noto Sans', system-ui, sans-serif";
    
    applyEffects(ctx);

    const cTitle = theme.palette[theme.colors["text-main"]] || "#ffffff";
    const sTitle = theme.typography["font-size-title"];
    const wTitle = theme.typography["font-weight-title"];
    
    ctx.fillStyle = cTitle;
    ctx.font = `${wTitle} ${sTitle}px ${fontStack}`;
    ctx.textBaseline = "middle"; 
    
    const titleY = lhTitle / 2;
    const titleText = fitText(ctx, album.title, w);
    ctx.fillText(titleText, 0, titleY);
    
    const cArtist = theme.palette[theme.colors["text-muted"]] || "#cccccc";
    const sArtist = theme.typography["font-size-artist"];
    const wArtist = theme.typography["font-weight-artist"];
    
    ctx.fillStyle = cArtist;
    ctx.font = `${wArtist} ${sArtist}px ${fontStack}`;
    
    const artistY = lhTitle + gapLesser + (lhArtist / 2);
    const artistText = fitText(ctx, album.artist, w);
    ctx.fillText(artistText, 0, artistY);
  }

  $effect(() => {
    const _ = {
      t: album.title, 
      a: album.artist, 
      w: coverSize, 
      h: textBlockHeight,
      c1: theme.colors["text-main"],
      c2: theme.colors["text-muted"],
      bg: theme.colors["background-main"],
      dpr: window.devicePixelRatio 
    };
    renderText();
  });
</script>

<div class="album-unit">
  <button 
    class="album-cover" 
    class:active
    style="z-index: 10;"
    {onclick}
    aria-label="Select album {album.title}"
  >
    {#if coverUrl}
      <img 
        src={coverUrl} 
        alt="" 
        decoding="async"
        draggable="false"
      />
    {/if}
  </button>
  
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
        display: block;
        image-rendering: -webkit-optimize-contrast;
      "
    ></canvas>
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
    border-radius: 0px;
    box-shadow: var(--album-cover-shadow);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
    pointer-events: auto;
    overflow: hidden;
  }

  .album-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .album-info {
    display: block;
    position: relative;
    will-change: opacity, clip-path;
  }
</style>
