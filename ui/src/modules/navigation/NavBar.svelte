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
    width: 52px;
    height: 100%;
    background-color: var(--background-drawer);
    border-left: 1px solid rgba(255, 255, 255, 0.05);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 0;
    box-sizing: border-box;
    z-index: 300;
    flex-shrink: 0;
  }
  
  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
