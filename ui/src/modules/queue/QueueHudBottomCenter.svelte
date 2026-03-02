<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";

  let tickingElapsed = $state(0);

  let progress = $derived(
    player.duration > 0 
      ? (tickingElapsed / player.duration) * 100 
      : 0
  );

  function tick() {
    if (player.state === "play") {
      const delta = (performance.now() - player.lastUpdated) / 1000;
      tickingElapsed = Math.min(player.elapsed + delta, player.duration);
    } else {
      tickingElapsed = player.elapsed;
    }
    requestAnimationFrame(tick);
  }

  onMount(() => {
    const raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });
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
    padding: 0 40px;
    box-sizing: border-box;
  }

  .progress-track {
    position: relative;
    width: 100%;
    height: 2px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 1px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: #fff;
    box-shadow: 0 0 8px rgba(255, 255, 255, 0.5);
    will-change: width;
  }
</style>
