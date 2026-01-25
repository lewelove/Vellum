import { connectSocket } from "./api.js";
import LogicWorker from "./workers/logic.worker.js?worker"; // Vite Worker Import

class LibraryState {
  // --- UI STATE ---
  
  // The Reactive View (The Sorted List displayed by the Grid)
  albums = $state([]); 
  
  // Sidebar State
  sidebarGroups = $state(new Map()); // key -> items[]
  
  // Status
  isLoading = $state(true);
  isConnected = $state(false);
  
  // Controls
  expandedAlbumId = $state(null);
  activeFilter = $state({ key: null, val: null });
  
  // activeSort determines the current view's order
  activeSort = $state({ key: "default" });
  
  // userSortPreference stores the user's choice for the Media Library view
  userSortPreference = $state("default");
  
  activeSidebarGrouper = $state("genre");

  // Signal to reset scroll position
  viewVersion = $state(0);

  // --- INTERNAL ARCHITECTURE ---

  // The Heavy Cache (Not reactive, just storage)
  // Map<string, AlbumObject>
  // Stores the actual 10KB objects to prevent constant transfer overhead.
  albumCache = new Map();

  worker = null;
  
  // Instrumentation
  _tRequest = 0;
  _pendingViewReset = false;

  init() {
    // 1. Initialize Worker
    this.worker = new LogicWorker();
    
    this.worker.onmessage = (e) => {
      const { type, data, ids, timing, result, key, count } = e.data;

      // A. Initial Population
      if (type === "INIT_DATA") {
        console.log(`📥 [Main] Caching ${count} objects...`);
        // Populate the Vault
        data.forEach(a => this.albumCache.set(a.id, a));
        // Refresh sidebar now that data is available
        this.refreshSidebar();
      }
      
      // B. Single Update (Hot Reload)
      else if (type === "UPDATE_DATA") {
        this.albumCache.set(data.id, data);
      }

      // C. View Update (The "Fast" Part)
      else if (type === "VIEW_UPDATED") {
        const tTransferEnd = performance.now();
        const transferTime = (tTransferEnd - this._tRequest - parseFloat(timing)).toFixed(2);
        
        // Hydrate IDs -> Objects
        // Map lookup is O(1). This loop is extremely fast.
        this.albums = ids.map(id => this.albumCache.get(id)).filter(Boolean);
        
        this.isLoading = false;

        // Reset scroll if filter/sort changed
        if (this._pendingViewReset) {
            this.viewVersion++;
            this._pendingViewReset = false;
        }

        console.log(`⏱️ [Worker] Logic: ${timing} ms | Transfer: ${transferTime} ms | Items: ${ids.length}`);
      } 
      
      // D. Sidebar Update
      else if (type === "GROUP_RESULT") {
        const newMap = new Map(this.sidebarGroups);
        newMap.set(key, result);
        this.sidebarGroups = newMap;
      }
    };

    // 2. Connect to WebSocket
    connectSocket(
      () => { this.isConnected = true; },
      (event) => this.handleSocketMessage(event)
    );
  }

  handleSocketMessage(event) {
    if (event.data instanceof Blob) {
      const reader = new FileReader();
      reader.onload = () => {
        try {
          const json = JSON.parse(reader.result);
          
          // ROUTING LOGIC: Correctly identify if message is a full INIT or a single UPDATE
          // This prevents hot reloads from wiping the library state.
          if (json.type === "UPDATE") {
            this.worker.postMessage({ type: "UPDATE", payload: json.payload });
          } else if (json.type === "INIT") {
            this.worker.postMessage({ type: "INIT", payload: json.data || json });
          }
        } catch (err) {
          console.error("Failed to parse binary websocket message:", err);
        }
      };
      reader.readAsText(event.data);
    } else {
      // String Fallback
      try {
        const json = JSON.parse(event.data);
        if (json.type === "UPDATE") {
          this.worker.postMessage({ type: "UPDATE", payload: json.payload });
        } else if (json.type === "INIT") {
          this.worker.postMessage({ type: "INIT", payload: json.data || json });
        }
      } catch (err) {
        console.error("Failed to parse string websocket message:", err);
      }
    }
  }

  // --- PUBLIC ACTIONS ---

  refreshView(resetScroll = true) {
    if (!this.worker) return;
    this._tRequest = performance.now(); // Start timer
    this._pendingViewReset = resetScroll;
    
    // Snapshot state to prevent mutation during message passing
    this.worker.postMessage({
      type: "PROCESS",
      payload: {
        filter: $state.snapshot(this.activeFilter),
        sort: $state.snapshot(this.activeSort)
      }
    });
  }

  refreshSidebar() {
    if (!this.worker) return;
    this.worker.postMessage({ 
      type: "GROUP", 
      payload: { key: this.activeSidebarGrouper } 
    });
  }

  // --- UI BINDINGS ---

  getSidebarGroup(key) {
    // If cache miss, request computation
    if (!this.sidebarGroups.has(key) && this.worker) {
        this.worker.postMessage({ type: "GROUP", payload: { key } });
        return [];
    }
    return this.sidebarGroups.get(key) || [];
  }

  setSidebarGrouper(key) {
    this.activeSidebarGrouper = key;
    this.refreshSidebar();
  }

  /**
   * Primary Navigation Method.
   * Applying a filter implies entering a specific view, which should always
   * respect the user's Global Sort Preference.
   */
  applyFilter(key, val) {
    if (this.activeFilter.key === key && this.activeFilter.val === val) {
      this.activeFilter = { key: null, val: null };
    } else {
      this.activeFilter = { key, val };
    }
    this.expandedAlbumId = null;
    
    // Enforce Global Sort Preference
    this.activeSort = { key: this.userSortPreference };
    
    this.refreshView(true);
  }

  /**
   * Atomic Action: Switch to "Recently Added" view.
   * Clears filters and forces date-based sorting override.
   */
  showRecentlyAdded() {
    this.activeFilter = { key: null, val: null };
    this.activeSort = { key: "date_added" };
    this.expandedAlbumId = null;
    this.refreshView(true);
  }

  /**
   * Atomic Action: Switch to "Media Library" view.
   * Clears filters and restores user preference sorting.
   */
  showMediaLibrary() {
    this.activeFilter = { key: null, val: null };
    this.activeSort = { key: this.userSortPreference };
    this.expandedAlbumId = null;
    this.refreshView(true);
  }

  /**
   * Applies a temporary sort override (e.g., manually triggered)
   * Does NOT persist to user preference.
   */
  applySort(key) {
    this.activeSort = { key };
    this.refreshView(true);
  }

  /**
   * Sets the user's global library sort preference.
   * Updates the view immediately.
   */
  setUserSort(key) {
    this.userSortPreference = key;
    this.activeSort = { key };
    this.refreshView(true);
  }

  /**
   * Restores the user's preferred sort.
   * Kept for compatibility, though showMediaLibrary() is preferred for navigation.
   */
  restoreUserSort() {
    this.activeSort = { key: this.userSortPreference };
    this.refreshView(true);
  }

  toggleExpand(id) {
    this.expandedAlbumId = (this.expandedAlbumId === id) ? null : id;
  }
}

export const library = new LibraryState();
