<script>
  import QueueNavButton from "./QueueNavButton.svelte";

  let { panels, onToggle } = $props();
</script>

<div class="queue-bar">
  <div class="nav-group">
    <QueueNavButton 
      icon="icons/24px/format_list_bulleted.svg" 
      label="Track List" 
      active={panels.tracks}
      onclick={() => onToggle('tracks')} 
    />
    <QueueNavButton 
      icon="icons/24px/lyrics.svg" 
      label="Lyrics" 
      active={panels.lyrics}
      onclick={() => onToggle('lyrics')} 
    />
  </div>
</div>

<style>
  .queue-bar {
    height: 100%;
    background-color: rgba(36, 36, 36, 0.66);
    backdrop-filter: blur(8px);
    border-left: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 0 16px rgba(0, 0, 0, 0.1), 0 0 16px rgba(0, 0, 0, 0.2), 0 0 10px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px;
    box-sizing: border-box;
    z-index: 100;
    flex-shrink: 0;
  }
  
  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
