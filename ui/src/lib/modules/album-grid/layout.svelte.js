import { theme } from "../../../theme.svelte.js";

export class LayoutManager {
  containerWidth = $state(0);
  
  gap = $derived(theme.layout["grid-gap-main"]);
  cardSize = $derived(theme.layout["grid-cover-size"]);
  
  rowHeight = $derived(
    theme.layout["grid-gap-main"] +       
    theme.layout["grid-cover-size"] +     
    theme.layout["grid-text-gap-main"] +  
    theme.layout["font-size-title"] +     
    theme.layout["grid-text-gap-lesser"] +
    theme.layout["font-size-artist"]
  );

  cols = $derived(Math.floor((Math.max(0, this.containerWidth - 40) + this.gap) / (this.cardSize + this.gap)) || 1);

  gridWidth = $derived(Math.floor((this.cols * this.cardSize) + ((this.cols - 1) * this.gap)));

  chunk(arr) {
    const results = [];
    for (let i = 0; i < arr.length; i += this.cols) {
      results.push(arr.slice(i, i + this.cols));
    }
    return results;
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
