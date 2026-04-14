import { theme } from "../../../theme.svelte.js";

export class LayoutManager {
  containerWidth = $state(0);

  gapX = $derived(theme.albumGrid["gap-x"] ?? 24);
  gapY = $derived(theme.albumGrid["gap-y"] ?? 12);
  cardSize = $derived(theme.albumGrid["cover-size"] ?? 200);
  
  creaseHeight = $derived(theme.albumGrid["crease-height"] ?? 0);
  
  rowHeight = $derived(
    this.gapY +       
    this.cardSize +     
    (theme.albumGrid["text-gap-main"] ?? 8) +  
    (theme.albumGrid["font-line-height-title"] ?? 18) +     
    (theme.albumGrid["text-gap-lesser"] ?? 2) +
    (theme.albumGrid["font-line-height-artist"] ?? 16)
  );

  cols = $derived(Math.max(1, Math.floor((this.containerWidth - 40 + this.gapX) / (this.cardSize + this.gapX))));
  gridWidth = $derived((this.cols * this.cardSize) + ((this.cols - 1) * this.gapX));

  get topOffset() {
    return this.creaseHeight - this.gapY;
  }

  getTotalHeight(rowCount) {
    return (rowCount * this.rowHeight) + this.topOffset;
  }

  getRowY(index) {
    return (index * this.rowHeight) + this.topOffset;
  }

  getVisibleIndices(scrollY, viewportHeight, rowCount) {
    const buffer = 2;
    const start = Math.floor(scrollY / this.rowHeight) - buffer;
    const end = Math.ceil((scrollY + viewportHeight) / this.rowHeight) + buffer;
    
    return {
      start: Math.max(0, start),
      end: Math.min(Math.max(0, rowCount - 1), end)
    };
  }

  chunk(arr) {
    const results = [];
    const columns = this.cols;
    for (let i = 0; i < arr.length; i += columns) {
      results.push(arr.slice(i, i + columns));
    }
    return results;
  }
}
