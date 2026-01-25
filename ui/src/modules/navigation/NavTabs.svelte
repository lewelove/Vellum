<script>
  import NavButton from "./NavButton.svelte";
</script>

<div class="nav-tabs-container">
  <NavButton icon="house.svg" tab="home" />
  <NavButton icon="queue.svg" tab="queue" />
</div>

<style>
  .nav-tabs-container {
    display: flex;
    flex-direction: row;
    justify-content: center;
    align-items: center;
    gap: 12px;
    width: 100%;
    height: 100%;
    background-color: var(--background-drawer);
    box-sizing: border-box;
  }
</style>
