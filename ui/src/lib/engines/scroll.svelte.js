export class ScrollEngine {
  currentY = $state(0);
  targetIndex = $state(0);
  wheelAccumulator = 0;
  
  constructor(damping = 0.12, threshold = 40) {
    this.damping = damping;
    this.threshold = threshold;
  }

  update(rowElements) {
    let targetY = 0;
    if (rowElements[this.targetIndex]) {
      targetY = rowElements[this.targetIndex].offsetTop - 20;
    }
    
    if (targetY < 0) targetY = 0;

    const diff = targetY - this.currentY;
    if (Math.abs(diff) > 0.5) {
      this.currentY += diff * this.damping;
    } else {
      this.currentY = targetY;
    }
  }

  handleWheel(e, maxRows) {
    this.wheelAccumulator += e.deltaY;
    if (Math.abs(this.wheelAccumulator) > this.threshold) {
      const direction = this.wheelAccumulator > 0 ? 1 : -1;
      this.targetIndex = Math.max(0, Math.min(this.targetIndex + direction, maxRows - 1));
      this.wheelAccumulator = 0;
    }
  }
}
