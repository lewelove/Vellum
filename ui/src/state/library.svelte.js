import { getLibrary } from "$core/api.js";

class LibraryState {
  albums = $state([]);
  expandedAlbumId = $state(null);
  isLoading = $state(false);

  async load() {
    this.isLoading = true;
    try {
      // The backend now returns the full Custodian payload:
      // Albums with tracks already folded inside.
      this.albums = await getLibrary();
    } catch (e) {
      console.error("Failed to load library:", e);
    } finally {
      this.isLoading = false;
    }
  }

  toggleExpand(id) {
    // Pure UI state toggle. Data is already there.
    if (this.expandedAlbumId === id) {
      this.expandedAlbumId = null;
    } else {
      this.expandedAlbumId = id;
    }
  }
}

export const library = new LibraryState();
