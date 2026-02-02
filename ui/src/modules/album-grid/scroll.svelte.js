export class ScrollEngine {
  currentY = $state(0);
  targetSlot = $state(0); 
  wheelAccumulator = 0;
  
  constructor(damping = 0.14, threshold = 40) {
    this.damping = damping;
    this.threshold = threshold;
  }

  update(rowHeight, dpr = 1) {
    // 1. Calculate the Ideal Target based on logical slots
    const idealTargetY = this.targetSlot * rowHeight;
    
    // 2. Quantize the Target to the Physical Grid
    // We snap the destination so the spring comes to rest on a hardware integer.
    // This prevents the "soft blur" resting state.
    const physicalTargetY = Math.round(idealTargetY * dpr) / dpr;

    // 3. Calculate Physics on High-Precision Floats
    // We do NOT quantize the delta or the velocity during transit.
    // The "blur" during motion is acceptable/expected; 
    // the "judder" from quantizing motion is not.
    const diff = physicalTargetY - this.currentY;
    
    if (Math.abs(diff) < 0.005) {
      this.currentY = physicalTargetY;
    } else {
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

  shiftPosition(deltaY, rowHeight, maxSlots) {
    this.currentY += deltaY;
    const slotDelta = deltaY / rowHeight;
    this.targetSlot = Math.max(0, Math.min(this.targetSlot + slotDelta, maxSlots));
  }
}
