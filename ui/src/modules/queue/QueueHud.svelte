<script>
  import QueueControl from "./QueueControl.svelte";

  let { children } = $props();
</script>

<div class="queue-hud">
  <main class="hud-main">
    {@render children()}
  </main>

  <QueueControl />
</div>

<style>
  .queue-hud {
    position: absolute;
    inset: 0;
    z-index: 100;
    display: flex;
    flex-direction: column;
    pointer-events: none;
  }

  .hud-main {
    flex: 1;
    width: 100%;
    position: relative;
    min-height: 0;
    padding: 16px 36px 0 36px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
  }
</style>
