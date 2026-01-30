<script>
  let { label, count, active, onclick } = $props();
</script>

<button class="sidebar-item" class:active {onclick}>
  <span class="label" title={label}>{label}</span>
  <span class="count">{count}</span>
</button>

<style>
  .sidebar-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    background: none;
    border: none;
    padding: 8px 24px;
    cursor: pointer;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 14px;
    text-align: left;
    transition: color 0.1s, background-color 0.1s;
    outline: none;
  }

  .sidebar-item:hover {
    color: var(--text-main);
    background-color: rgba(255, 255, 255, 0.03);
  }

  .sidebar-item.active {
    color: var(--text-main);
    background-color: rgba(255, 255, 255, 0.05);
  }

  .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-right: 8px;
  }

  .count {
    opacity: 0.5;
    font-size: 12px;
  }
</style>
