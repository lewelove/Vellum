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

  // --- BITMAP TEXT RASTERIZER ---
  
  let canvas;
  
  // Dimensions
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

  function renderText() {
    if (!canvas) return;
    
    const dpr = window.devicePixelRatio || 1;
    const w = coverSize;
    const h = textBlockHeight;

    if (w <= 0 || h <= 0) return;

    canvas.width = w * dpr;
    canvas.height = h * dpr;
    
    const ctx = canvas.getContext('2d', { alpha: true });
    ctx.scale(dpr, dpr);
    
    // Clear
    ctx.clearRect(0, 0, w, h);
    
    // Font Configuration
    const fontStack = "Inter, 'Noto Sans', system-ui, sans-serif";
    
    // 1. Title Layer
    const cTitle = theme.palette[theme.colors["text-main"]] || "#ffffff";
    const sTitle = theme.typography["font-size-title"];
    const wTitle = theme.typography["font-weight-title"];
    
    ctx.fillStyle = cTitle;
    ctx.font = `${wTitle} ${sTitle}px ${fontStack}`;
    ctx.textBaseline = "middle"; 
    
    // Position vertically centered within the first line height
    const titleY = lhTitle / 2;
    const titleText = fitText(ctx, album.title, w);
    ctx.fillText(titleText, 0, titleY);
    
    // 2. Artist Layer
    const cArtist = theme.palette[theme.colors["text-muted"]] || "#cccccc";
    const sArtist = theme.typography["font-size-artist"];
    const wArtist = theme.typography["font-weight-artist"];
    
    ctx.fillStyle = cArtist;
    ctx.font = `${wArtist} ${sArtist}px ${fontStack}`;
    
    // Position vertically centered within the second line height, offset by first line + gap
    const artistY = lhTitle + gapLesser + (lhArtist / 2);
    const artistText = fitText(ctx, album.artist, w);
    ctx.fillText(artistText, 0, artistY);
  }

  $effect(() => {
    // Reactive Trigger
    const _ = {
      t: album.title, 
      a: album.artist, 
      w: coverSize, 
      h: textBlockHeight,
      c1: theme.colors["text-main"],
      c2: theme.colors["text-muted"]
    };
    renderText();
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
      height: {textBlockHeight}px;
    "
  >
    <canvas 
      bind:this={canvas}
      style="
        width: {coverSize}px;
        height: {textBlockHeight}px;
        display: block;
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
    background-size: cover;
    background-position: center;
    border-radius: 0px;
    box-shadow: var(--album-cover-shadow);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
    pointer-events: auto;
  }

  .album-info {
    display: block;
    position: relative;
    will-change: opacity, clip-path;
  }
</style>
