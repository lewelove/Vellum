export class LayoutManager {
  containerWidth = $state(0);
  
  // SSOT METRICS
  cardSize = 200;
  gap = 20;       // This is the vertical space between rows
  textHeight = 40; // Title + Artist area
  
  // A Row Unit = Gap + Card + Text
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
   * Quantizes drawer to Row Units
   */
  getQuantizedDrawer(trackCount) {
    const headerHeight = 100; 
    const trackHeight = 40;   
    const naturalHeight = headerHeight + (trackCount * trackHeight) + 40; 
    
    const virtualRows = Math.ceil(naturalHeight / this.rowHeight);
    return {
      height: virtualRows * this.rowHeight,
      rows: virtualRows
    };
  }
}
