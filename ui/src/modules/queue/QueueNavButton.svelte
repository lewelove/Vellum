<script>
  let { icon, active, onclick, label } = $props();
</script>

<button 
  class="queue-nav-button" 
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

  .queue-nav-button:hover {
    background-color: rgba(255, 255, 255, 0.03);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .queue-nav-button.active {
    background-color: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.1);
    box-shadow: var(--button-shadow);
  }

  .nav-icon {
    opacity: 0.3;
    transition: opacity 0.2s ease;
    width: 28px;
    height: 28px;
  }

  .queue-nav-button:hover .nav-icon,
  .queue-nav-button.active .nav-icon {
    opacity: 1;
  }
</style>
