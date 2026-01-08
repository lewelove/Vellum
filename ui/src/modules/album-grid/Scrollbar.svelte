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

<style>
  .custom-scrollbar-track {
    position: absolute;
    top: 0;
    right: 0;
    width: 6px;
    height: 100%;
    background: transparent;
    z-index: 100;
  }

  .custom-scrollbar-thumb {
    position: absolute;
    top: 0;
    right: 0;
    width: 100%;
    background-color: rgba(255, 255, 255, 0.15);
    border-radius: 3px;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .custom-scrollbar-thumb:hover {
    background-color: rgba(255, 255, 255, 0.3);
  }
</style>
