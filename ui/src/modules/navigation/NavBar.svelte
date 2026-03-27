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
    background-color: #24242442;
    backdrop-filter: blur(16px) brightness(0.7);
  }
  
  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
