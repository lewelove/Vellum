import { connectSocket } from "$core/api.js";
import { applyFilter } from "../logic/filters.js";
import { sorters } from "../logic/sorters.js";
import { generateSidebarGroup } from "../logic/groupers.js";

class LibraryState {
  rawAlbums = $state([]); // Reactive Source of Truth
  
  // Reactive UI State
  isLoading = $state(true);
  isConnected = $state(false);
  expandedAlbumId = $state(null);
  
  activeFilter = $state({ key: null, val: null });
  activeSort = $state({ key: "default" });
  activeSidebarGrouper = $state("genre");

  // Signal to reset scroll position (incremented on view changes)
  viewVersion = $state(0);

  // 1. FILTERING ENGINE
  filteredAlbums = $derived.by(() => {
    if (!this.activeFilter.key) return this.rawAlbums;
    return this.rawAlbums.filter(a => applyFilter(a, this.activeFilter.key, this.activeFilter.val));
  });

  // 2. SORTING ENGINE
  albums = $derived.by(() => {
    // Clone to avoid mutating filtered source during sort
    const list = [...this.filteredAlbums];
    
    const sorter = sorters[this.activeSort.key] || sorters.date_added;
    list.sort(sorter);

    return list;
  });

  capabilities = {
    grouping: ["genre", "decade"]
  };

  init() {
    connectSocket(
      () => { this.isConnected = true; },
      (event) => this.handleMessage(event)
    );
  }

  handleMessage(event) {
    if (event.data instanceof Blob) {
      // Handle binary if we sent binary, but we send text JSON
      const reader = new FileReader();
      reader.onload = () => {
        this.processPayload(JSON.parse(reader.result));
      };
      reader.readAsText(event.data);
    } else {
      this.processPayload(JSON.parse(event.data));
    }
  }

  processPayload(msg) {
    if (msg.type === "INIT") {
      this.processInit(msg.data);
    } else if (msg.type === "UPDATE") {
      this.processUpdate(msg.id, msg.payload);
    }
  }

  processInit(data) {
    // Enhance data for UI
    data.forEach(a => {
      a.title = a.ALBUM;
      a.artist = a.ALBUMARTIST;
    });
    
    // Atomic replacement of state
    this.rawAlbums = data;
    this.isLoading = false;
  }

  processUpdate(id, albumData) {
    // Enhance
    albumData.title = albumData.ALBUM;
    albumData.artist = albumData.ALBUMARTIST;

    const index = this.rawAlbums.findIndex(a => a.id === id);
    
    if (index !== -1) {
      // Replace existing
      // Svelte 5 array mutation is fine if using $state
      this.rawAlbums[index] = albumData;
    } else {
      // New Album
      this.rawAlbums.push(albumData);
    }
  }

  // Sidebar Logic
  getSidebarGroup(key) {
    return generateSidebarGroup(this.rawAlbums, key);
  }

  setSidebarGrouper(key) {
    this.activeSidebarGrouper = key;
  }

  applyFilter(key, val) {
    if (this.activeFilter.key === key && this.activeFilter.val === val) {
      this.activeFilter = { key: null, val: null };
    } else {
      this.activeFilter = { key, val };
    }
    this.expandedAlbumId = null;
    this.viewVersion++; // Trigger scroll reset
  }

  applySort(key) {
    this.activeSort = { key };
    this.viewVersion++; // Trigger scroll reset
  }

  toggleExpand(id) {
    this.expandedAlbumId = (this.expandedAlbumId === id) ? null : id;
  }
}

export const library = new LibraryState();
