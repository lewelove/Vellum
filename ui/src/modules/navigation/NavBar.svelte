<script>
  import NavButton from "./NavButton.svelte";
</script>

<nav class="nav-bar">
  <div class="nav-group top">
    <NavButton icon="icons/24px/home.svg" tab="home" />
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
  
  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
