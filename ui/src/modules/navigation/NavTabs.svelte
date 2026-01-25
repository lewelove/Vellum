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
    flex-direction: column;
    gap: 12px;
    padding: 24px 0;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }
</style>
