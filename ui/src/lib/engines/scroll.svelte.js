export class ScrollEngine {
  currentY = $state(0);
  targetSlot = $state(0); 
  wheelAccumulator = 0;
  
  constructor(damping = 0.16, threshold = 40) {
    this.damping = damping;
    this.threshold = threshold;
  }

  update(viewportHeight, contentHeight, rowHeight) {
    const targetY_ideal = this.targetSlot * rowHeight;
    
    // Boundary Clamping: We allow scroll up to Content + 1 Row (Hero Row)
    let targetY = targetY_ideal;
    const maxScroll = Math.max(0, (contentHeight + rowHeight) - viewportHeight);
    if (targetY > maxScroll) targetY = maxScroll;
    if (targetY < 0) targetY = 0;

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
