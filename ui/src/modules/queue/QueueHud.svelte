<script>
  import QueueHudTopLeft from "./QueueHudTopLeft.svelte";
  import QueueHudTopCenter from "./QueueHudTopCenter.svelte";
  import QueueHudTopRight from "./QueueHudTopRight.svelte";
  import QueueHudBottomLeft from "./QueueHudBottomLeft.svelte";
  import QueueHudBottomCenter from "./QueueHudBottomCenter.svelte";
  import QueueHudBottomRight from "./QueueHudBottomRight.svelte";

  let { children } = $props();
</script>

<div class="queue-hud">
  <header class="hud-bar top">
    <div class="corner left">
      <QueueHudTopLeft />
    </div>
    <div class="center">
      <QueueHudTopCenter />
    </div>
    <div class="corner right">
      <QueueHudTopRight />
    </div>
  </header>

  <main class="hud-main">
    {@render children()}
  </main>

  <footer class="hud-bar bottom">
    <div class="corner left">
      <QueueHudBottomLeft />
    </div>
    <div class="center">
      <QueueHudBottomCenter />
    </div>
    <div class="corner right">
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
    /* Updated background color */
    background-color: #242424;
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
    min-width: 200px;
  }

  .left {
    justify-content: flex-start;
  }

  .right {
    justify-content: flex-end;
  }

  .center {
    flex: 1;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: auto;
    min-width: 0;
  }

  .hud-main {
    flex: 1;
    width: 100%;
    position: relative;
    min-height: 0;
  }
</style>
