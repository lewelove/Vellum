export class ScrollEngine {
  currentY = $state(0);
  targetSlot = $state(0); 
  wheelAccumulator = 0;
  
  constructor(damping = 0.20, threshold = 40) {
    this.damping = damping;
    this.threshold = threshold;
  }

  update(rowHeight) {
    const targetY = this.targetSlot * rowHeight;
    const diff = targetY - this.currentY;

    if (Math.abs(diff) < 0.1) {
      this.currentY = targetY;
    } else {
      this.currentY += diff * this.damping;
    }
  }

  handleWheel(e, maxSlots) {
    this.wheelAccumulator += e.deltaY;
    
    if (Math.abs(this.wheelAccumulator) > this.threshold) {
      const direction = this.wheelAccumulator > 0 ? 1 : -1;
      this.targetSlot = Math.max(0, Math.min(this.targetSlot + direction, maxSlots));
      this.wheelAccumulator = 0;
    }
  }

  syncToSlot(slot) {
    this.targetSlot = slot;
  }
}
