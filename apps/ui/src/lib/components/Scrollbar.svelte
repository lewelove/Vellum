<script>
  let { currentY, viewportHeight, contentHeight } = $props();

  let scrollProgress = $derived(contentHeight > viewportHeight 
    ? Math.min(1, Math.max(0, currentY / (contentHeight - viewportHeight)))
    : 0);
    
  let thumbHeight = $derived(contentHeight > viewportHeight 
    ? Math.max(50, (viewportHeight / contentHeight) * viewportHeight) 
    : 0);
    
  let thumbY = $derived(scrollProgress * (viewportHeight - thumbHeight));
</script>

<div class="custom-scrollbar-track">
  <div 
    class="custom-scrollbar-thumb"
    style="height: {thumbHeight}px; transform: translateY({thumbY}px);"
  ></div>
</div>
