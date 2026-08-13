<script lang="ts">
import { view } from "../../library/view.svelte.ts";
import AlbumGrid from "./AlbumGrid/AlbumGrid.svelte";

let {
  activeAlbumId = null,
  onfocus = () => {}
}: {
  activeAlbumId?: string | null;
  onfocus?: (album: any) => void;
} = $props();
</script>

<AlbumGrid
  albums={view.libraryAlbums}
  version={view.libraryVersion}
  resetVersion={view.libraryResetVersion}
  {activeAlbumId}
  {onfocus}
/>
