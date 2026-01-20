import { getLibraryArtifact } from "$core/api.js";
import { applyFilter } from "../logic/filters.js";
import { sorters } from "../logic/sorters.js";
import { generateSidebarGroup } from "../logic/groupers.js";
import { theme } from "$core/theme.svelte.js";

// Helper to generate color (ported from Python)
function generateColor(id) {
  // Simple hash for color consistency
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = id.charCodeAt(i) + ((hash << 5) - hash);
  }
  const palette = ["#EA4335", "#34A853", "#FBBC04", "#4285F4", "#A142F4", "#F4426C", "#42F4E2"];
  const index = Math.abs(hash) % palette.length;
  return palette[index];
}

class LibraryState {
  rawTracks = []; // The Flat Lake (Immutable after load)
  
  // Reactive State
  isLoading = $state(true);
  expandedAlbumId = $state(null);
  
  activeFilter = $state({ key: null, val: null });
  activeSort = $state({ key: "date_added" });

  // 1. FILTERING ENGINE
  filteredTracks = $derived.by(() => {
    if (!this.activeFilter.key) return this.rawTracks;
    return this.rawTracks.filter(t => applyFilter(t, this.activeFilter.key, this.activeFilter.val));
  });

  // 2. GROUPING ENGINE (Tracks -> Albums)
  albums = $derived.by(() => {
    // Group tracks by album_id
    // Using a Map for O(N) performance
    const groups = new Map();
    
    for (const track of this.filteredTracks) {
      const aid = track.album_id;
      if (!groups.has(aid)) {
        groups.set(aid, {
          id: aid,
          title: track.ALBUM || "Unknown",
          artist: track.ALBUMARTIST || "Unknown",
          color: generateColor(aid),
          cover_path: track.cover_path, // From Flat Lake
          tracks: [],
          // For sorting convenience, cache aggregated values
          unix_added: track.unix_added
        });
      }
      groups.get(aid).tracks.push(track);
    }

    const albumList = Array.from(groups.values());

    // 3. SORTING ENGINE
    const sorter = sorters[this.activeSort.key] || sorters.date_added;
    albumList.sort(sorter);

    return albumList;
  });

  // Capabilities are now hardcoded in UI logic, 
  // but we expose them for the Sidebar to iterate.
  capabilities = {
    grouping: ["genre", "decade"]
  };

  async init() {
    this.isLoading = true;
    try {
      this.rawTracks = await getLibraryArtifact();
    } catch (e) {
      console.error("Library load failed", e);
    } finally {
      this.isLoading = false;
    }
  }

  // Sidebar Logic
  getSidebarGroup(key) {
    // Generate groups based on the *Entire* library (ignoring current filter usually, 
    // or respecting it depending on UX preference. Usually sidebar counts absolute library).
    return generateSidebarGroup(this.rawTracks, key);
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
