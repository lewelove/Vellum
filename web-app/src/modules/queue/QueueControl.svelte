<script>
  import { player } from "../player.svelte.js";

  let isPlaying = $derived(player.state === "play");
  
  async function togglePlay() {
    try { await fetch('/api/toggle-pause', { method: 'POST' }); } catch(e) {}
  }
  
  async function next() {
    try { await fetch('/api/next', { method: 'POST' }); } catch(e) {}
  }
  
  async function prev() {
    try { await fetch('/api/prev', { method: 'POST' }); } catch(e) {}
  }
</script>

<div class="queue-control-transport">
  <button class="ctrl-btn secondary" onclick={prev}>
    <img src="/icons/24px/skip_previous.svg" alt="Previous" />
  </button>
  <button class="ctrl-btn primary" onclick={togglePlay}>
    <img 
      src={isPlaying ? "/icons/24px/pause.svg" : "/icons/24px/play_arrow.svg"} 
      alt={isPlaying ? "Pause" : "Play"} 
    />
  </button>
  <button class="ctrl-btn secondary" onclick={next}>
    <img src="/icons/24px/skip_next.svg" alt="Next" />
  </button>
</div>

<style>
  .queue-control-transport {
    width: 100%;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 32px;
    box-sizing: border-box;
  }

  .ctrl-btn {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition: opacity 0.2s;
    opacity: 0.8;
  }

  .ctrl-btn:hover {
    opacity: 1;
  }

  .ctrl-btn img {
    width: 24px;
    height: 24px;
  }

  .ctrl-btn.primary img {
    width: 32px;
    height: 32px;
  }
</style>
