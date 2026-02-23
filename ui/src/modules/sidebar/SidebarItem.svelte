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
    background-color: transparent;
    border: none;
    padding: 6px 12px;
    margin-bottom: 2px;
    cursor: default;
    color: var(--text-muted);
    font-family: var(--font-stack);
    font-size: 14px;
    text-align: left;
    transition: background-color 0.1s ease;
    outline: none;
    border-radius: 8px;
    box-sizing: border-box;
    user-select: none;
    contain: layout;
    -webkit-font-smoothing: subpixel-antialiased;
  }

  .sidebar-item:hover {
    background-color: rgba(255, 255, 255, 0.03);
    color: var(--text-main);
    cursor: pointer;
  }

  .sidebar-item.active {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-main);
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
    font-size: 13px;
    font-feature-settings: "tnum";
    font-variant-numeric: tabular-nums;
  }
</style>
