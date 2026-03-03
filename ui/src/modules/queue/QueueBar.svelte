<script>
  import QueueNavButton from "./QueueNavButton.svelte";

  let { activeView, onViewChange } = $props();
</script>

<div class="queue-bar">
  <div class="nav-group">
    <QueueNavButton 
      icon="icons/24px/format_list_bulleted.svg" 
      label="Track List" 
      active={activeView === 'tracks'}
      onclick={() => onViewChange('tracks')} 
    />
    <QueueNavButton 
      icon="icons/24px/lyrics.svg" 
      label="Lyrics" 
      active={activeView === 'lyrics'}
      onclick={() => onViewChange('lyrics')} 
    />
  </div>
</div>

<style>
  .queue-bar {
    height: 100%;
    background-color: var(--background-drawer);
    box-shadow: var(--album-cover-shadow);
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
