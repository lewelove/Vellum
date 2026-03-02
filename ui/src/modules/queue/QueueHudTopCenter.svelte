<script>
  import { player } from '$lib/player';
</script>

<div class="queue-hud-top-center">
  {#if player.track}
    <div class="metadata-wrapper">
      <span class="artist">
        {player.track.album_artist || player.track.artist || 'Unknown Artist'}
      </span>
      <span class="separator">:</span>
      <span class="album">
        {player.track.album || 'Unknown Album'}
      </span>
    </div>
  {/if}
</div>

<style>
  .queue-hud-top-center {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .metadata-wrapper {
    display: flex;
    gap: 0.5rem;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-primary, #ffffff);
    white-space: nowrap;
    opacity: 0.8;
  }

  .separator {
    opacity: 0.4;
  }

  .artist,
  .album {
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
