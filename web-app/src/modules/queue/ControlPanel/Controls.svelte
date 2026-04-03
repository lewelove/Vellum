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
  <button class="ctrl-btn" onclick={prev}>
    <img src="/icons/24px/skip_previous.svg" alt="Previous" />
  </button>
  <button class="ctrl-btn main" onclick={togglePlay}>
    <img src={isPlaying ? "/icons/24px/pause.svg" : "/icons/24px/play_arrow.svg"} alt="Toggle Play" />
  </button>
  <button class="ctrl-btn" onclick={next}>
    <img src="/icons/24px/skip_next.svg" alt="Next" />
  </button>
</div>
