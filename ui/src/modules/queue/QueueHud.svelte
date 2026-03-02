<script>
  import QueueHudTopLeft from "./QueueHudTopLeft.svelte";
  import QueueHudTopRight from "./QueueHudTopRight.svelte";
  import QueueHudBottomLeft from "./QueueHudBottomLeft.svelte";
  import QueueHudBottomRight from "./QueueHudBottomRight.svelte";

  let { children } = $props();
</script>

<div class="queue-hud">
  <header class="hud-bar top">
    <div class="corner">
      <QueueHudTopLeft />
    </div>
    <div class="corner">
      <QueueHudTopRight />
    </div>
  </header>

  <main class="hud-main">
    {@render children()}
  </main>

  <footer class="hud-bar bottom">
    <div class="corner">
      <QueueHudBottomLeft />
    </div>
    <div class="corner">
      <QueueHudBottomRight />
    </div>
  </footer>
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

  .hud-bar {
    height: 48px;
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    background-color: transparent;
    pointer-events: none;
    box-sizing: border-box;
    flex-shrink: 0;
  }

  .top {
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .bottom {
    border-top: 1px solid rgba(255, 255, 255, 0.05);
  }

  .corner {
    height: 100%;
    display: flex;
    align-items: center;
    pointer-events: auto;
  }

  .hud-main {
    flex: 1;
    width: 100%;
    position: relative;
    min-height: 0;
  }
</style>
