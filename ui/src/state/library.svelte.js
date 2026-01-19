import { getLibrary, getCapabilities, getSidebarGroup } from "$core/api.js";

class LibraryState {
  albums = $state([]);
  expandedAlbumId = $state(null);
  isLoading = $state(false);

  // Capabilities (What features the server has)
  capabilities = $state({
    grouping: [],
    filtering: [],
    sorting: []
  });

  // Active View State
  activeSort = $state({ key: "date_added", dir: "DESC" });
  activeFilter = $state({ key: null, val: null });
  
  // Cache for sidebar data to prevent refetching
  sidebarCache = new Map();

  async init() {
    try {
      this.capabilities = await getCapabilities();
      this.load();
    } catch (e) {
      console.error("Init error:", e);
    }
  }

  async load() {
    this.isLoading = true;
    try {
      const params = {
        sort: this.activeSort.key,
        // The API defaults to DESC, but explicit is better
        // Note: Our API logic currently infers DIR from params or default, 
        // we map it here conceptually.
      };
      
      // If our sort direction logic in UI becomes dynamic, pass it
      // params.sort_dir = this.activeSort.dir;

      if (this.activeFilter.key && this.activeFilter.val) {
        params.filter = this.activeFilter.key;
        params.val = this.activeFilter.val;
      }

      this.albums = await getLibrary(params);
    } catch (e) {
      console.error("Failed to load library:", e);
    } finally {
      this.isLoading = false;
    }
  }

  // Called by SidebarSection
  async fetchSidebarGroup(key) {
    if (this.sidebarCache.has(key)) {
      return this.sidebarCache.get(key);
    }
    try {
      const data = await getSidebarGroup(key);
      this.sidebarCache.set(key, data);
      return data;
    } catch (e) {
      console.error("Group fetch error:", e);
      return [];
    }
  }

  toggleExpand(id) {
    this.expandedAlbumId = (this.expandedAlbumId === id) ? null : id;
  }
  
  applyFilter(key, val) {
    // If clicking the same filter, clear it (Toggle behavior)
    if (this.activeFilter.key === key && this.activeFilter.val === val) {
      this.activeFilter = { key: null, val: null };
    } else {
      this.activeFilter = { key, val };
    }
    this.expandedAlbumId = null; 
    this.load();
  }

  applySort(key) {
    // Simple toggle logic could go here, for now just set key
    this.activeSort.key = key;
    this.expandedAlbumId = null;
    this.load();
  }
}

export const library = new LibraryState();
