export class ScrollEngine {
  currentY = $state(0);
  targetIndex = $state(0);
  wheelAccumulator = 0;
  
  constructor(damping = 0.16, threshold = 40) {
    this.damping = damping;
    this.threshold = threshold;
  }

  update(rowElements, viewportHeight, contentHeight) {
    // If we have no rows yet, stay still
    if (!rowElements || rowElements.length === 0) return;
    
    // Safety check for targetIndex
    const safeIndex = Math.max(0, Math.min(this.targetIndex, rowElements.length - 1));
    const targetRow = rowElements[safeIndex];
    
    // CRITICAL: If the row exists but has no offsetParent, it's not yet rendered properly
    if (!targetRow || targetRow.offsetTop === undefined) return;

    let targetY = targetRow.offsetTop - 20;
    
    // Guard: Only clamp to contentHeight if contentHeight is valid (> 0)
    // and if we aren't in the middle of a massive layout shift
    if (contentHeight > viewportHeight) {
      const maxScroll = Math.max(0, contentHeight - viewportHeight);
      targetY = Math.min(targetY, maxScroll);
    }
    
    if (targetY < 0) targetY = 0;

    const diff = targetY - this.currentY;

    // Use a slightly larger epsilon for the snapping to prevent "vibrating" on resize
    if (Math.abs(diff) < 0.5) {
      this.currentY = targetY;
    } else {
      this.currentY += diff * this.damping;
    }
  }

  handleWheel(e, maxAllowedRows) {
    if (maxAllowedRows <= 0) return;
    
    this.wheelAccumulator += e.deltaY;
    if (Math.abs(this.wheelAccumulator) > this.threshold) {
      const direction = this.wheelAccumulator > 0 ? 1 : -1;
      this.targetIndex = Math.max(0, Math.min(this.targetIndex + direction, maxAllowedRows - 1));
      this.wheelAccumulator = 0;
    }
  }
}
