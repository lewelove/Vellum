import { getLibraryArtifact } from "$core/api.js";
import { applyFilter } from "../logic/filters.js";
import { sorters } from "../logic/sorters.js";
import { generateSidebarGroup } from "../logic/groupers.js";
import { theme } from "$core/theme.svelte.js";

function generateColor(id) {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = id.charCodeAt(i) + ((hash << 5) - hash);
  }
  const palette = ["#121212"];
  const index = Math.abs(hash) % palette.length;
  return palette[index];
}

class LibraryState {
  rawAlbums = []; // The Nested Lake
  
  // Reactive State
  isLoading = $state(true);
  expandedAlbumId = $state(null);
  
  activeFilter = $state({ key: null, val: null });
  activeSort = $state({ key: "date_added" });

  // 1. FILTERING ENGINE
  filteredAlbums = $derived.by(() => {
    if (!this.activeFilter.key) return this.rawAlbums;
    return this.rawAlbums.filter(a => applyFilter(a, this.activeFilter.key, this.activeFilter.val));
  });

  // 2. SORTING ENGINE
  albums = $derived.by(() => {
    // Clone to avoid mutating filtered source during sort
    const list = [...this.filteredAlbums];
    
    // Inject Colors (if not present) - UI Enhancement
    // We do this lazily or here. Since data is fresh from JSON, let's just ensure colors exist.
    // In a production app, we might bake this in the build step, but this is fast enough.
    list.forEach(a => {
        if (!a.color) a.color = generateColor(a.id);
        // Map keys to UI expectations if necessary (though our JSON keys mostly match)
        a.title = a.ALBUM;
        a.artist = a.ALBUMARTIST;
    });

    const sorter = sorters[this.activeSort.key] || sorters.date_added;
    list.sort(sorter);

    return list;
  });

  capabilities = {
    grouping: ["genre", "decade"]
  };

  async init() {
    this.isLoading = true;
    try {
      this.rawAlbums = await getLibraryArtifact();
    } catch (e) {
      console.error("Library load failed", e);
    } finally {
      this.isLoading = false;
    }
  }

  // Sidebar Logic
  getSidebarGroup(key) {
    return generateSidebarGroup(this.rawAlbums, key);
  }

  applyFilter(key, val) {
    if (this.activeFilter.key === key && this.activeFilter.val === val) {
      this.activeFilter = { key: null, val: null };
    } else {
      this.activeFilter = { key, val };
    }
    this.expandedAlbumId = null;
  }

  toggleExpand(id) {
    this.expandedAlbumId = (this.expandedAlbumId === id) ? null : id;
  }
}

export const library = new LibraryState();
