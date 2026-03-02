<script>
  import { nav, setTab } from "../../navigation.svelte.js";

  let { icon, tab } = $props();
  let isActive = $derived(nav.activeTab === tab);
</script>

<button 
  class="nav-button" 
  class:active={isActive} 
  class:queue-mode={nav.activeTab === 'queue'}
  onclick={() => setTab(tab)}
  title={tab}
>
  <img src="/icons/{icon}.svg" alt={tab} class="nav-icon" />
</button>

<style>
  .nav-button {
    width: 36px;
    height: 36px;
    background-color: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: 
      background-color 0.2s, 
      border-color 0.2s,
      transform 0.1s;
    pointer-events: auto;
    border-radius: 10px;
    box-shadow: var(--button-shadow-lesser);
    flex-shrink: 0;
  }

  .nav-button:hover {
    background-color: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .nav-button.active {
    background-color: var(--text-main);
    border-color: var(--text-main);
  }

  .nav-icon {
    width: 24px;
    height: 24px;
    filter: invert(0.8);
    transition: filter 0.2s;
  }

  .nav-button.active .nav-icon {
    filter: invert(0);
  }

  .nav-button.queue-mode:not(.active) {
    background-color: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .nav-button.queue-mode.active {
    background-color: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .nav-button.queue-mode.active .nav-icon {
    filter: invert(1);
    opacity: 1;
  }
</style>
