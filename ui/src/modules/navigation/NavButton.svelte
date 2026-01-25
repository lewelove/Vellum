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
  <img src="/icons/{icon}" alt={tab} class="nav-icon" />
</button>

<style>
  .nav-button {
    width: 48px;
    height: 48px;
    background-color: var(--palette-100);
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color 0.2s, transform 0.1s;
    /* Squircle Clip Path */
    clip-path: path('M 0,24 C 0,4.8 4.8,0 24,0 C 43.2,0 48,4.8 48,24 C 48,43.2 43.2,48 24,48 C 4.8,48 0,43.2 0,24 Z');
  }

  .nav-button:hover {
    background-color: var(--palette-300);
  }

  .nav-button.active {
    background-color: var(--text-main);
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
</style>
