import { connectSocket } from "$core/api.js";
import LogicWorker from "../workers/logic.worker.js?worker"; // Vite Worker Import

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
  activeSort = $state({ key: "default" });
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
        // We use .filter(Boolean) just in case of race conditions (e.g. ID exists but cache missing)
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
        // Send RAW STRING to worker.
        // Worker will perform JSON.parse() off the main thread.
        this.worker.postMessage({ type: "INIT", payload: reader.result });
      };
      reader.readAsText(event.data);
    } else {
      // Hot Reload Update
      const json = JSON.parse(event.data);
      if (json.type === "UPDATE") {
         this.worker.postMessage({ type: "UPDATE", payload: json.payload });
      } else if (json.type === "INIT") {
         // Fallback for non-binary init
         this.worker.postMessage({ type: "INIT", payload: json.data });
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

  applyFilter(key, val) {
    if (this.activeFilter.key === key && this.activeFilter.val === val) {
      this.activeFilter = { key: null, val: null };
    } else {
      this.activeFilter = { key, val };
    }
    this.expandedAlbumId = null;
    this.refreshView(true);
  }

  applySort(key) {
    this.activeSort = { key };
    this.refreshView(true);
  }

  toggleExpand(id) {
    this.expandedAlbumId = (this.expandedAlbumId === id) ? null : id;
  }
}

export const library = new LibraryState();
