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
  <img src="/icons/{icon}" alt={tab} class="nav-icon" />
</button>

<style>
  .nav-button {
    width: 48px;
    height: 48px;
    background-color: var(--background-drawer);
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color 0.2s, transform 0.1s, box-shadow 0.2s;
    pointer-events: auto;
    border-radius: 12px;
  }

  .nav-button.queue-mode {
    background-color: transparent;
    box-shadow: none;
  }

  .nav-button:hover {
    background-color: var(--palette-300);
  }

  .nav-button.queue-mode:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  .nav-button.active {
    background-color: var(--text-main);
  }

  .nav-button.queue-mode.active {
    background-color: rgba(255, 255, 255, 0.2);
  }

  .nav-icon {
    width: 22px;
    height: 22px;
    filter: invert(0.8);
    transition: filter 0.2s;
  }

  .nav-button.active .nav-icon {
    filter: invert(0.1);
  }

  .nav-button.queue-mode .nav-icon {
    filter: invert(1);
    opacity: 0.8;
  }

  .nav-button.queue-mode.active .nav-icon {
    opacity: 1;
  }
</style>
