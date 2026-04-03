<script>
  import { player } from "../../player.svelte.js";

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

<div class="controls">
  <button class="v-btn-icon control-btn" onclick={prev} title="Previous">
    <img src="/icons/24px/skip_previous.svg" alt="" />
  </button>
  <button class="v-btn-icon control-btn" onclick={togglePlay} title="Toggle Play">
    <img src={isPlaying ? "/icons/24px/pause.svg" : "/icons/24px/play_arrow.svg"} alt="" />
  </button>
  <button class="v-btn-icon control-btn" onclick={next} title="Next">
    <img src="/icons/24px/skip_next.svg" alt="" />
  </button>
</div>

<style>
  .controls {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .control-btn {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    flex-shrink: 0;
  }

  .control-btn img {
    width: 24px;
    height: 24px;
  }
</style>
