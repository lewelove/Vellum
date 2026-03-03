<script>
  import { nav, setTab } from "../../navigation.svelte.js";

  let { icon, tab } = $props();
  let isActive = $derived(nav.activeTab === tab);
</script>

<button 
  class="nav-button" 
  class:active={isActive} 
  onclick={() => setTab(tab)}
  title={tab}
>
  <img src="/{icon}" alt={tab} class="nav-icon" />
</button>

<style>
  .nav-button {
    width: 40px;
    height: 40px;
    background-color: rgba(255, 255, 255, 0.01);
    border: 2px solid rgba(255, 255, 255, 0.08);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    pointer-events: auto;
    border-radius: 10px;
    box-shadow: var(--button-shadow-lesser);
    flex-shrink: 0;
  }

  .nav-button:hover {
    background-color: rgba(255, 255, 255, 0.03);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .nav-button.active {
    background-color: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.1);
    box-shadow: var(--button-shadow);
  }

  .nav-icon {
    width: 28px;
    height: 28px;
    opacity: 0.3;
    transition: opacity 0.2s ease;
  }

  .nav-button:hover .nav-icon,
  .nav-button.active .nav-icon {
    opacity: 1;
  }
</style>
