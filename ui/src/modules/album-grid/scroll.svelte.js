export class ScrollEngine {
  currentY = $state(0);
  targetSlot = $state(0); 
  wheelAccumulator = 0;
  
  constructor(damping = 0.14, threshold = 40) {
    this.damping = damping;
    this.threshold = threshold;
  }

  update(rowHeight, dpr = 1) {
    // 1. Calculate the ideal target
    const idealTargetY = this.targetSlot * rowHeight;
    
    // 2. SNAP the target itself to the physical pixel grid
    // This ensures the "destination" is something the monitor can actually render
    const snappedTargetY = Math.round(idealTargetY * dpr) / dpr;
    
    const diff = snappedTargetY - this.currentY;

    // 3. Increased threshold. 
    // If the difference is less than 0.1 of a CSS pixel, 
    // it's visually indistinguishable from the target. arrive now.
    if (Math.abs(diff) < 0.1) {
      this.currentY = snappedTargetY;
    } else {
      // 4. Smooth glide towards the already-snapped target
      this.currentY += diff * this.damping;
    }
  }

  handleWheel(e, maxSlots) {
    this.wheelAccumulator += e.deltaY;
    
    if (Math.abs(this.wheelAccumulator) > this.threshold) {
      const direction = this.wheelAccumulator > 0 ? 1 : -1;
      const base = Math.round(this.targetSlot);
      
      this.targetSlot = Math.max(0, Math.min(base + direction, maxSlots));
      this.wheelAccumulator = 0;
    }
  }

  syncToSlot(slot) {
    this.targetSlot = slot;
  }
}
