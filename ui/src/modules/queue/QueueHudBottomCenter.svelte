<script>
  import { player } from '$lib/stores/player';

  $: progress = ($player.position / $player.duration) * 100 || 0;
</script>

<div class="queue-hud-bottom-center">
  <div class="progress-track">
    <div 
      class="progress-fill" 
      style:width="{progress}%"
    ></div>
  </div>
</div>

<style>
  .queue-hud-bottom-center {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    padding: 0 1.5rem;
  }

  .progress-track {
    position: relative;
    width: 100%;
    height: 2px;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 1px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent-color, #ffffff);
    transition: width 0.2s linear;
  }
</style>
