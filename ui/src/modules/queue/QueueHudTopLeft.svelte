<script>
  import { setTab } from "../../navigation.svelte.js";
</script>

<div class="hud-corner-unit">
  <button class="nav-back" onclick={() => setTab('home')}>
  </button>
</div>

<style>
  .hud-corner-unit {
    padding: 0 20px;
    height: 100%;
    display: flex;
    align-items: center;
  }

  .nav-back {
    display: flex;
    align-items: center;
    gap: 12px;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    opacity: 0.8;
    transition: opacity 0.2s;
  }

  .nav-back:hover {
    opacity: 1;
  }

  .icon {
    width: 18px;
    height: 18px;
    filter: invert(1);
    transform: rotate(180deg);
  }

  .hud-line {
    color: #fff;
    font-size: 14px;
    letter-spacing: 0.08em;
    font-family: var(--font-mono);
  }
</style>
