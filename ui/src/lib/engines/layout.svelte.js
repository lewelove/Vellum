export class LayoutManager {
  containerWidth = $state(0);
  
  // FIXED METRICS (The SSOT)
  cardSize = 200;
  gap = 20;
  textHeight = 60; // Space for Title + Artist + Margins inside Album component
  
  // Derived Row Height: The atomic unit of travel
  rowHeight = $derived(this.cardSize + this.gap + this.textHeight);

  cols = $derived(Math.floor((Math.max(0, this.containerWidth - 40) + this.gap) / (this.cardSize + this.gap)) || 1);
  gridWidth = $derived(Math.floor((this.cols * this.cardSize) + ((this.cols - 1) * this.gap)));

  chunk(arr) {
    const results = [];
    for (let i = 0; i < arr.length; i += this.cols) {
      results.push(arr.slice(i, i + this.cols));
    }
    return results;
  }

  /**
   * Quantizes drawer height to virtual rows
   */
  getQuantizedDrawer(trackCount) {
    const headerHeight = 100; 
    const trackHeight = 41;   
    const naturalHeight = headerHeight + (trackCount * trackHeight) + 40; 
    
    const virtualRows = Math.ceil(naturalHeight / this.rowHeight);
    return {
      height: virtualRows * this.rowHeight,
      rows: virtualRows
    };
  }
}
