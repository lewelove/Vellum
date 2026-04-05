<script>
  import NavButton from "./NavButton.svelte";

  let { variant = "solid" } = $props();
</script>

<nav class="nav-bar" class:v-glass={variant === 'glass'}>
  <div class="nav-group top">
    <NavButton icon="icons/24px/house.svg" tab="home" />
    <NavButton icon="icons/24px/queue_music.svg" tab="queue" />
  </div>
</nav>

<style>
  .nav-bar {
    height: 100%;
    background-color: var(--background-drawer);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px;
    box-sizing: border-box;
    box-shadow: var(--panel-shadow);
    z-index: 100;
    flex-shrink: 0;
  }

  .nav-bar.v-glass {
    /* background-color: oklch(0% 0 0 / 0.3) !important; */
  }
  
  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
