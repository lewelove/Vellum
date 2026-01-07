import { getLibrary } from "$lib/api.js";

class LibraryState {
  albums = $state([]);
  expandedAlbumId = $state(null);
  isLoading = $state(false);

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

  toggleExpand(id) {
    this.expandedAlbumId = this.expandedAlbumId === id ? null : id;
  }
}

export const library = new LibraryState();
