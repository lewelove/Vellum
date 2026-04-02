<script>
  import NavButton from "./NavButton.svelte";
</script>

<div class="nav-tabs-container">
  <NavButton icon="icons/24px/home.svg" tab="home" />
  <NavButton icon="icons/24px/queue_music.svg" tab="queue" />
</div>

<style>
  .nav-tabs-container {
    display: flex;
    flex-direction: row;
    justify-content: flex-start;
    align-items: center;
    gap: 8px;
    box-sizing: border-box;
    pointer-events: auto;
  }
</style>
