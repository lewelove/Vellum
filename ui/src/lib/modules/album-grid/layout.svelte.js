import { theme } from "../../../theme.svelte.js";

export class LayoutManager {
  containerWidth = $state(0);
  
  paddingTop = $derived(theme["album-grid"]["crease-height"]);
  gapX = $derived(theme["album-grid"]["gap-x"]);
  gapY = $derived(theme["album-grid"]["gap-y"]);
  cardSize = $derived(theme["album-grid"]["cover-size"]);
  
  rowHeight = $derived(
    theme["album-grid"]["gap-y"] +       
    theme["album-grid"]["cover-size"] +     
    theme["album-grid"]["text-gap-main"] +  
    theme.typography["font-size-title"] +     
    theme["album-grid"]["text-gap-lesser"] +
    theme.typography["font-size-artist"]
  );

  cols = $derived(Math.floor((Math.max(0, this.containerWidth - 40) + this.gapX) / (this.cardSize + this.gapX)) || 1);

  gridWidth = $derived(Math.floor((this.cols * this.cardSize) + ((this.cols - 1) * this.gapX)));

  chunk(arr) {
    const results = [];
    for (let i = 0; i < arr.length; i += this.cols) {
      results.push(arr.slice(i, i + this.cols));
    }
    return results;
  }

  getContentHeight(totalRowsCount) {
    return (totalRowsCount * this.rowHeight) + this.paddingTop;
  }

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
