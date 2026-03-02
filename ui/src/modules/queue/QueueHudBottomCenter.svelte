<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";

  let tickingElapsed = $state(0);

  let progress = $derived(
    player.duration > 0 
      ? (tickingElapsed / player.duration) * 100 
      : 0
  );

  const formatTime = (totalSeconds) => {
    const s = Math.floor(totalSeconds || 0);
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const rs = s % 60;
    const pad = (num) => String(num).padStart(2, '0');
    if (h > 0) return `${h}:${pad(m)}:${pad(rs)}`;
    return `${m}:${pad(rs)}`;
  };

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
  <span class="time-label left">
    {formatTime(tickingElapsed)}
  </span>

  <div class="progress-track-wrapper">
    <div class="progress-track">
      <div 
        class="progress-fill" 
        style:width="{progress}%"
      ></div>
    </div>
  </div>

  <span class="time-label right">
    {formatTime(player.duration)}
  </span>
</div>

<style>
  .queue-hud-bottom-center {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    padding: 0 20px;
    box-sizing: border-box;
    gap: 16px;
  }

  .time-label {
    color: #fff;
    font-size: 13px;
    min-width: 45px;
    opacity: 0.8;
    font-family: var(--font-mono);
  }

  .left {
    text-align: right;
  }

  .right {
    text-align: left;
  }

  .progress-track-wrapper {
    flex: 1;
    display: flex;
    align-items: center;
    min-width: 0;
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
