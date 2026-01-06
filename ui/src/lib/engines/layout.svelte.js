export class LayoutManager {
  containerWidth = $state(0);
  cardSize = 200;
  gap = 20;

  cols = $derived(Math.floor((Math.max(0, this.containerWidth - 40) + this.gap) / (this.cardSize + this.gap)) || 1);
  gridWidth = $derived(Math.floor((this.cols * this.cardSize) + ((this.cols - 1) * this.gap)));

  chunk(arr) {
    const results = [];
    for (let i = 0; i < arr.length; i += this.cols) {
      results.push(arr.slice(i, i + this.cols));
    }
    return results;
  }
}
