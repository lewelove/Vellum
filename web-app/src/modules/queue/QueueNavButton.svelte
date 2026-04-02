<script>
  let { icon, active, onclick, label } = $props();
</script>

<button 
  class="v-btn-icon queue-nav-button" 
  class:active 
  {onclick}
  title={label}
>
  <img src="/{icon}" alt={label} class="nav-icon" />
</button>

<style>
  .queue-nav-button {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    box-shadow: var(--button-shadow-lesser);
    flex-shrink: 0;
    pointer-events: auto;
  }

  .nav-icon {
    width: 24px;
    height: 24px;
  }
</style>
