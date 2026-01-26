import { connectSocket } from "./api.js";
import { player, updatePlayerState } from "./modules/player.svelte.js"; // Import Player Store
import LogicWorker from "./workers/logic.worker.js?worker"; 

class LibraryState {

  albums = $state([]); 
  
  sidebarGroups = $state(new Map()); 
  
  isLoading = $state(true);
  isConnected = $state(false);
  
  expandedAlbumId = $state(null);
  activeFilter = $state({ key: null, val: null });
  
  activeSort = $state({ key: "default" });
  
  userSortPreference = $state("default");
  
  activeSidebarGrouper = $state("genre");

  viewVersion = $state(0);

  albumCache = new Map();
  trackPathMap = new Map();

  worker = null;
  
  _tRequest = 0;
  _pendingViewReset = false;

  init() {
    this.worker = new LogicWorker();
    
    this.worker.onmessage = (e) => {
      const { type, data, ids, timing, result, key, count } = e.data;

      if (type === "INIT_DATA") {
        console.log(`[Main] Caching ${count} objects...`);
        this.trackPathMap.clear();
        data.forEach(a => {
          this.albumCache.set(a.id, a);
          if (a.tracks) {
            a.tracks.forEach(t => {
              if (t.track_library_path) {
                this.trackPathMap.set(t.track_library_path, t);
              }
            });
          }
        });
        this.refreshSidebar();
      }
      
      else if (type === "UPDATE_DATA") {
        this.albumCache.set(data.id, data);
        if (data.tracks) {
          data.tracks.forEach(t => {
            if (t.track_library_path) {
              this.trackPathMap.set(t.track_library_path, t);
            }
          });
        }
      }

      else if (type === "VIEW_UPDATED") {
        const tTransferEnd = performance.now();
        const transferTime = (tTransferEnd - this._tRequest - parseFloat(timing)).toFixed(2);
        
        this.albums = ids.map(id => this.albumCache.get(id)).filter(Boolean);
        
        this.isLoading = false;

        if (this._pendingViewReset) {
            this.viewVersion++;
            this._pendingViewReset = false;
        }

        console.log(`[Worker] Logic: ${timing} ms | Transfer: ${transferTime} ms | Items: ${ids.length}`);
      } 
      
      else if (type === "GROUP_RESULT") {
        const newMap = new Map(this.sidebarGroups);
        newMap.set(key, result);
        this.sidebarGroups = newMap;
      }
    };

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
          this.dispatchSocketAction(json);
        } catch (err) {
          console.error("Failed to parse binary websocket message:", err);
        }
      };
      reader.readAsText(event.data);
    } else {
      try {
        const json = JSON.parse(event.data);
        this.dispatchSocketAction(json);
      } catch (err) {
        console.error("Failed to parse string websocket message:", err);
      }
    }
  }

  dispatchSocketAction(json) {
    if (json.type === "UPDATE") {
      this.worker.postMessage({ type: "UPDATE", payload: json.payload });
    } else if (json.type === "INIT") {
      this.worker.postMessage({ type: "INIT", payload: json.data || json });
    } else if (json.type === "MPD_STATUS") {
      updatePlayerState(json);
    }
  }

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

  getSidebarGroup(key) {
    if (!this.sidebarGroups.has(key) && this.worker) {
        this.worker.postMessage({ type: "GROUP", payload: { key } });
        return [];
    }
    return this.sidebarGroups.get(key) || [];
  }

  getTrackByPath(path) {
    return this.trackPathMap.get(path);
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
    
    this.activeSort = { key: this.userSortPreference };
    
    this.refreshView(true);
  }

  showRecentlyAdded() {
    this.activeFilter = { key: null, val: null };
    this.activeSort = { key: "date_added" };
    this.expandedAlbumId = null;
    this.refreshView(true);
  }

  showMediaLibrary() {
    this.activeFilter = { key: null, val: null };
    this.activeSort = { key: this.userSortPreference };
    this.expandedAlbumId = null;
    this.refreshView(true);
  }

  applySort(key) {
    this.activeSort = { key };
    this.refreshView(true);
  }

  setUserSort(key) {
    this.userSortPreference = key;
    this.activeSort = { key };
    this.refreshView(true);
  }

  restoreUserSort() {
    this.activeSort = { key: this.userSortPreference };
    this.refreshView(true);
  }

  toggleExpand(id) {
    this.expandedAlbumId = (this.expandedAlbumId === id) ? null : id;
  }
}

export const library = new LibraryState();
