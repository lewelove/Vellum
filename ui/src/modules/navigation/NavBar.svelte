<script>
  import NavButton from "./NavButton.svelte";

  let { variant = "solid" } = $props();
</script>

<nav class="nav-bar" class:glass={variant === 'glass'}>
  <div class="nav-group top">
    <NavButton icon="icons/24px/house.svg" tab="home" />
    <NavButton icon="icons/24px/queue_music.svg" tab="queue" />
  </div>
</nav>

<style>
  .nav-bar {
    height: 100%;
    background-color: var(--background-drawer);
    box-shadow: var(--album-cover-shadow);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px;
    box-sizing: border-box;
    z-index: 100;
    flex-shrink: 0;
  }

  .nav-bar.glass {
    background-color: rgba(36, 36, 36, 0.66);
    backdrop-filter: blur(30px);
    border-right: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 0 16px rgba(0, 0, 0, 0.1), 0 0 16px rgba(0, 0, 0, 0.2), 0 0 10px rgba(0, 0, 0, 0.2);
  }
  
  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
