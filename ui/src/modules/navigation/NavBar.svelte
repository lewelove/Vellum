<script>
  import { onMount } from "svelte";
  import NavButton from "./NavButton.svelte";
  import { player } from "../player.svelte.js";

  // -- Playback Logic Merged from QueueHudBottomCenter --
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

<nav class="nav-bar">
  <div class="nav-section left">
    <NavButton icon="icons/24px/home.svg" tab="home" />
    <NavButton icon="icons/24px/queue_music.svg" tab="queue" />
  </div>

  <div class="nav-section center">
    <span class="time-label left-align">
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
  
    <span class="time-label right-align">
      {formatTime(player.duration)}
    </span>
  </div>

  <div class="nav-section right">
    <!-- Future controls -->
  </div>
</nav>

<style>
  .nav-bar {
    width: 100%;
    /* Height determined by content + padding */
    height: auto; 
    background-color: var(--background-drawer);
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    display: flex;
    flex-direction: row;
    align-items: center;
    /* 12px padding top/bottom */
    padding: 0px 16px; 
    box-sizing: border-box;
    z-index: 300;
    flex-shrink: 0;
    gap: 32px;
  }
  
  .nav-section {
    display: flex;
    align-items: center;
  }

  .left {
    gap: 12px;
    width: 200px;
    justify-content: flex-start;
  }

  .right {
    gap: 12px;
    width: 200px;
    justify-content: flex-end;
  }

  .center {
    flex: 1;
    gap: 16px;
    justify-content: center;
    min-width: 0;
  }

  .time-label {
    color: #fff;
    font-size: 13px;
    min-width: 45px;
    opacity: 0.8;
    font-family: var(--font-mono);
  }

  .left-align { text-align: right; }
  .right-align { text-align: left; }

  .progress-track-wrapper {
    flex: 1;
    display: flex;
    align-items: center;
    min-width: 0;
    max-width: 600px;
  }

  .progress-track {
    position: relative;
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: #fff;
    box-shadow: 0 0 8px rgba(255, 255, 255, 0.5);
    will-change: width;
    transition: width 0.1s linear;
  }
</style>
