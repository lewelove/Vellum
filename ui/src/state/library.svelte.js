import { connectSocket } from "$core/api.js";
import LogicWorker from "../workers/logic.worker.js?worker"; // Vite Worker Import

class LibraryState {
  // UI State (Result from Worker)
  albums = $state([]); 
  
  isLoading = $state(true);
  isConnected = $state(false);
  expandedAlbumId = $state(null);
  
  // Controls
  activeFilter = $state({ key: null, val: null });
  activeSort = $state({ key: "default" });
  activeSidebarGrouper = $state("genre");
  
  // Sidebar Data (Async)
  sidebarGroups = $state(new Map()); // key -> items[]

  viewVersion = $state(0);
  worker = null;

  init() {
    // 1. Start Worker
    this.worker = new LogicWorker();
    
    this.worker.onmessage = (e) => {
      const { type, data, timing, result, key } = e.data;

      if (type === "VIEW_UPDATED") {
        console.log(`⏱️ [Worker] Logic: ${timing} ms | Transfer: ${(performance.now() - this._tRequest).toFixed(2)} ms`);
        this.albums = data;
        this.isLoading = false;
        
        // Reset scroll on view change
        if (this._pendingViewReset) {
            this.viewVersion++;
            this._pendingViewReset = false;
        }
      } 
      else if (type === "READY") {
        console.log("Worker initialized with", e.data.count, "albums");
        // Trigger initial sort
        this.refreshView(false);
        // Load initial sidebar
        this.refreshSidebar();
      }
      else if (type === "GROUP_RESULT") {
        // Update sidebar cache
        const newMap = new Map(this.sidebarGroups);
        newMap.set(key, result);
        this.sidebarGroups = newMap;
      }
    };

    // 2. Connect Socket
    connectSocket(
      () => { this.isConnected = true; },
      (event) => this.handleSocketMessage(event)
    );
  }

  handleSocketMessage(event) {
    // Pass raw data directly to worker
    // If it's a blob, we read text then pass. 
    // If it's string (Hot Reload), pass directly.
    
    if (event.data instanceof Blob) {
      const reader = new FileReader();
      reader.onload = () => {
        // Send RAW STRING to worker (Parsing happens off-thread)
        this.worker.postMessage({ type: "INIT", payload: reader.result });
      };
      reader.readAsText(event.data);
    } else {
      const json = JSON.parse(event.data);
      if (json.type === "INIT") {
         // Should not happen with current server logic but safety check
         this.worker.postMessage({ type: "INIT", payload: json.data });
      } else if (json.type === "UPDATE") {
         this.worker.postMessage({ type: "UPDATE", payload: json.payload });
      }
    }
  }

  // --- API ---

  _tRequest = 0;
  _pendingViewReset = false;

  refreshView(resetScroll = true) {
    if (!this.worker) return;
    this._tRequest = performance.now();
    this._pendingViewReset = resetScroll;
    
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

  // --- UI Bindings ---

  getSidebarGroup(key) {
    // Trigger calc if missing (async)
    if (!this.sidebarGroups.has(key) && this.worker) {
        this.worker.postMessage({ type: "GROUP", payload: { key } });
        return []; // Return empty while loading
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
