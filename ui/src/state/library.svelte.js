import { getLibrary } from "$core/api.js";

class LibraryState {
  albums = $state([]);
  expandedAlbumId = $state(null);
  isLoading = $state(false);

  // View State (Filters & Sorting)
  view = $state({
    sortCol: "date_added",
    sortDir: "DESC",
    filterCol: null,
    filterVal: null
  });

  async load() {
    this.isLoading = true;
    try {
      const params = {
        sort_col: this.view.sortCol,
        sort_dir: this.view.sortDir
      };
      
      if (this.view.filterCol && this.view.filterVal) {
        params.filter_col = this.view.filterCol;
        params.filter_val = this.view.filterVal;
      }

      // Backend returns Pixel-Ready DTO (Albums with tracks included)
      this.albums = await getLibrary(params);
    } catch (e) {
      console.error("Failed to load library:", e);
    } finally {
      this.isLoading = false;
    }
  }

  // Sidebar Logic: Fetch Groups (e.g., list of Genres)
  async getGroups(key) {
    try {
      return await getLibrary({ group_by: key });
    } catch (e) {
      console.error("Failed to fetch groups:", e);
      return [];
    }
  }

  toggleExpand(id) {
    if (this.expandedAlbumId === id) {
      this.expandedAlbumId = null;
    } else {
      this.expandedAlbumId = id;
    }
  }
  
  applyFilter(col, val) {
    this.view.filterCol = col;
    this.view.filterVal = val;
    this.expandedAlbumId = null; // Collapse drawer on view change
    this.load();
  }

  applySort(col, dir = "ASC") {
    this.view.sortCol = col;
    this.view.sortDir = dir;
    this.expandedAlbumId = null;
    this.load();
  }
}

export const library = new LibraryState();
