# Feature: Pica-Rendered Canvas Cover Layered on Top of IMG Tag in Modal Drawer

## Overview

Ensure that the pica-rendered canvas cover is displayed on top of the regular `<img>` tag cover in the Modal Drawer, matching the same layering pattern used in QueueView.

## Problem Statement

### Current Behavior

**QueueView** (`ui/src/modules/queue/QueueView.svelte`, lines 126-154):
- Has a layered cover rendering approach:
  1. `hard-shadow` div with `<img>` inside (z-index: 1) - appears first
  2. `backing-image` `<img>` (z-index: 2) - appears after shadow
  3. `raw-canvas` with pica rendering (z-index: 3) - appears on top when ready
- The canvas fades in on top of the img tags once loaded

**ModalDrawerCover** (`ui/src/modules/album-grid/ModalDrawerCover.svelte`, lines 63-70):
- Only has a canvas element
- No underlying `<img>` tag as fallback/placeholder
- Canvas shows/hides based on `isLoaded` state

### Issue

The Modal Drawer is missing the layered approach - it doesn't have an underlying `<img>` tag that shows immediately while the pica-rendered canvas is loading. This creates an inconsistent user experience compared to QueueView.

## Proposed Solution

Add an `<img>` tag as a fallback/background layer in ModalDrawerCover, with the canvas rendered on top. This matches QueueView's pattern:

1. Show `<img>` tag immediately for fast initial render
2. Load pica-rendered canvas in background
3. Fade canvas in on top once ready

## Technical Details

### Files to Modify

- `ui/src/modules/album-grid/ModalDrawerCover.svelte`

### Implementation

Add an `<img>` element alongside the canvas:

```svelte
<div class="modal-drawer-cover-wrapper" style="width: {width}px; height: {height}px;">
  <!-- Background img layer - shows immediately -->
  <img 
    {src} 
    class="cover-image"
    alt=""
  />
  <!-- Foreground canvas - pica rendered, fades in when ready -->
  <canvas 
    bind:this={canvasEl} 
    class="output-canvas" 
    class:visible={isLoaded}
    style="width: {width}px; height: {height}px;"
  ></canvas>
</div>
```

### CSS Requirements

- `cover-image` should be behind canvas (lower z-index)
- `cover-image` should always be visible (no opacity transition)
- `output-canvas` should have opacity transition (0.4s) and higher z-index

## Acceptance Criteria

- [ ] ModalDrawerCover has an `<img>` tag showing the cover immediately
- [ ] Canvas renders with pica on top of the img
- [ ] Canvas fades in (0.4s transition) when ready
- [ ] Behavior matches QueueView layering pattern
- [ ] No regression in modal drawer cover rendering

## Related Components

- `ui/src/modules/album-grid/ModalDrawerCover.svelte`
- `ui/src/modules/queue/QueueView.svelte` (reference implementation)
