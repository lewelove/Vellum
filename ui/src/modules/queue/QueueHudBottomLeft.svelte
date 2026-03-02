<script>
  import NavTabs from "../navigation/NavTabs.svelte";
</script>

<div class="hud-corner-unit">
  <div class="hud-nav-group">
    <NavTabs />
  </div>
</div>

<style>
  .hud-corner-unit {
    padding: 0 20px;
    height: 100%;
    display: flex;
    align-items: center;
  }

  .hud-nav-group {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }
</style>
