import { getLibrary, getAlbumTracks } from "$core/api.js";

class LibraryState {
  albums = $state([]);
  expandedAlbumId = $state(null);
  isLoading = $state(false);
  
  // Cache to prevent re-fetching tracks
  trackCache = new Map();

  async load() {
    this.isLoading = true;
    try {
      this.albums = await getLibrary();
    } catch (e) {
      console.error("Failed to load library:", e);
    } finally {
      this.isLoading = false;
    }
  }

  async toggleExpand(id) {
    if (this.expandedAlbumId === id) {
      this.expandedAlbumId = null;
    } else {
      this.expandedAlbumId = id;
      await this.loadTracksIfNeeded(id);
    }
  }

  async loadTracksIfNeeded(id) {
    // 1. Check if we already have tracks in the album object
    const album = this.albums.find(a => a.id === id);
    if (!album) return;

    if (album.tracks && album.tracks.length > 0) return;

    // 2. Fetch
    try {
      const tracks = await getAlbumTracks(id);
      
      // 3. Update State (Reactivity triggers here)
      // Using .map just to be safe about rendering specific fields
      // The API returns fully inflated objects, so we can just assign strings
      album.tracks = tracks.map(t => t.TITLE || "Untitled"); 
    } catch (e) {
      console.error("Failed to fetch tracks for album", id, e);
    }
  }
}

export const library = new LibraryState();
