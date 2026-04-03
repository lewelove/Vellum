<script>
  import { onMount } from "svelte";
  import { player } from "../../player.svelte.js";

  let tickingElapsed = $state(0);
  let duration = $derived(player.duration || 0);
  let progress = $derived(duration > 0 ? (tickingElapsed / duration) * 100 : 0);

  function formatTime(totalSeconds) {
    const s = Math.floor(totalSeconds || 0);
    const m = Math.floor(s / 60);
    const rs = s % 60;
    const pad = (n) => String(n).padStart(2, '0');
    return `${m}:${pad(rs)}`;
  }

  function tick() {
    if (player.state === "play") {
      const delta = (performance.now() - player.lastUpdated) / 1000;
      tickingElapsed = Math.min(player.elapsed + delta, player.duration || 0);
    } else {
      tickingElapsed = player.elapsed || 0;
    }
    requestAnimationFrame(tick);
  }

  onMount(() => {
    const raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="progress-wrapper">
  <span class="v-mono time">{formatTime(tickingElapsed)}</span>
  <div class="track-container">
    <div class="progress-track">
      <div class="progress-fill" style="width: {progress}%"></div>
    </div>
  </div>
  <span class="v-mono time">{formatTime(duration)}</span>
</div>
