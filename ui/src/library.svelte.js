import { connectSocket } from "./api.js";
import { player, updatePlayerState } from "./modules/player.svelte.js";
import { nav } from "./navigation.svelte.js";
import LogicWorker from "./workers/logic.worker.js?worker"; 

class LibraryState {

  albums = $state([]); 
  sidebarGroups = $state(new Map()); 
  isLoading = $state(true);
  isConnected = $state(false);
  focusedAlbum = $state(null);
  activeFilter = $state({ key: null, val: null });
  activeSort = $state({ key: "default", order: "default" });
  userSortPreference = $state("default");
  userSortOrder = $state("default");
  activeSidebarGrouper = $state("genre");
  viewVersion = $state(0);
  albumCache = $state(new Map());
  trackPathMap = $state(new Map());
  pinnedTextures = $state(new Map());

  worker = null;
  _tRequest = 0;
  _pendingViewReset = false;

  init() {
    this.worker = new LogicWorker();
    
    this.worker.onmessage = (e) => {
      const { type, data, ids, timing, result, key, count } = e.data;

      if (type === "INIT_DATA") {
        console.log(`[Main] Caching ${count} objects...`);
        
        const newTrackMap = new Map();
        const newAlbumCache = new Map();

        data.forEach(a => {
          newAlbumCache.set(a.id, a);
          if (a.tracks) {
            a.tracks.forEach(t => {
              // Flatten track info
              if (t.info) Object.assign(t, t.info);
              
              // Normalize track keys
              t.TITLE = t.TITLE || t.title;
              t.ARTIST = t.ARTIST || t.artist;
              t.TRACKNUMBER = t.TRACKNUMBER || t.tracknumber;
              t.DISCNUMBER = t.DISCNUMBER || t.discnumber;
              
              t.ALBUMARTIST = a.ALBUMARTIST;
              t.album_id = a.id;
              if (t.track_library_path) {
                newTrackMap.set(t.track_library_path, t);
              }
            });
          }
        });

        this.trackPathMap = newTrackMap;
        this.albumCache = newAlbumCache;

        this.refreshSidebar();
        this.orchestratePrewarming(data);
      }
      
      else if (type === "UPDATE_DATA") {
        this.albumCache.set(data.id, data);
        if (data.tracks) {
          data.tracks.forEach(t => {
            if (t.info) Object.assign(t, t.info);
            t.TITLE = t.TITLE || t.title;
            t.ARTIST = t.ARTIST || t.artist;
            
            t.ALBUMARTIST = data.ALBUMARTIST;
            t.album_id = data.id;
            if (t.track_library_path) {
              this.trackPathMap.set(t.track_library_path, t);
            }
          });
        }
        if (this.focusedAlbum?.id === data.id) {
          this.focusedAlbum = data;
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
      if (json.ui_state) {
          this.applyPersistedState(json.ui_state);
      }
      this.worker.postMessage({ 
        type: "INIT", 
        payload: {
          data: json.data,
          ui_state: json.ui_state
        }
      });
    } else if (json.type === "MPD_STATUS") {
      updatePlayerState(json);
    }
  }

  async orchestratePrewarming(albumData) {
    const concurrencyLimit = 6;
    const queue = [...albumData];
    
    const processor = async () => {
      while (queue.length > 0) {
        const album = queue.shift();
        const url = this.getThumbnailUrl(album);
        
        if (!url || this.pinnedTextures.has(url)) continue;

        const img = new Image();
        img.src = url;
        
        try {
          await img.decode();
          this.pinnedTextures.set(url, img);
          this.pinnedTextures = new Map(this.pinnedTextures);
        } catch (err) {}
      }
    };

    const workers = Array.from({ length: concurrencyLimit }, () => processor());
    await Promise.all(workers);
    console.log(`[Prewarmer] Pinned ${this.pinnedTextures.size} textures.`);
  }

  applyPersistedState(state) {
      nav.activeTab = state.activeTab || "home";
      this.userSortPreference = state.sortKey || "default";
      this.userSortOrder = state.sortOrder || "default";
      this.activeSort = { key: this.userSortPreference, order: this.userSortOrder };
      this.activeSidebarGrouper = state.groupKey || "genre";
      this.activeFilter = state.filter || { key: null, val: null };
  }

  persistState() {
      fetch("/api/state", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
              activeTab: nav.activeTab,
              sortKey: this.userSortPreference,
              sortOrder: this.userSortOrder,
              groupKey: this.activeSidebarGrouper,
              filter: $state.snapshot(this.activeFilter)
          })
      }).catch(err => console.error("Failed to persist state:", err));
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

  getThumbnailUrl(album) {
    if (!album || !album.cover_hash) return "";
    return `/api/covers/${album.cover_hash}`;
  }

  getAlbumCoverUrl(albumId) {
    const album = this.albumCache.get(albumId);
    // Note: check flattened cover_path or nested info.cover_path
    const cp = album.cover_path || album.info?.cover_path;
    const ch = album.cover_hash || album.info?.cover_hash;
    if (!album || !cp || cp === "default_cover.png") {
      return "";
    }
    return `/api/assets/cover/${encodeURIComponent(album.id)}?v=${ch}`;
  }

  setSidebarGrouper(key) {
    this.activeSidebarGrouper = key;
    this.refreshSidebar();
    this.persistState();
  }

  applyFilter(key, val) {
    if (this.activeFilter.key === key && this.activeFilter.val === val) {
      this.activeFilter = { key: null, val: null };
    } else {
      this.activeFilter = { key, val };
    }
    this.focusedAlbum = null;
    this.activeSort = { key: this.userSortPreference, order: this.userSortOrder };
    this.refreshView(true);
    this.persistState();
  }

  showRecentlyAdded() {
    this.activeFilter = { key: null, val: null };
    this.activeSort = { key: "date_added", order: "default" }; 
    this.focusedAlbum = null;
    this.refreshView(true);
    this.persistState();
  }

  showMediaLibrary() {
    this.activeFilter = { key: null, val: null };
    this.activeSort = { key: this.userSortPreference, order: this.userSortOrder };
    this.focusedAlbum = null;
    this.refreshView(true);
    this.persistState();
  }

  applySort(key) {
    this.activeSort = { key, order: "default" };
    this.refreshView(true);
  }

  setUserSort(key) {
    this.userSortPreference = key;
    this.activeSort = { key, order: this.userSortOrder };
    this.refreshView(true);
    this.persistState();
  }

  toggleSortOrder() {
    this.userSortOrder = (this.userSortOrder === "default") ? "reverse" : "default";
    this.activeSort = { key: this.userSortPreference, order: this.userSortOrder };
    this.refreshView(true);
    this.persistState();
  }

  restoreUserSort() {
    this.activeSort = { key: this.userSortPreference, order: this.userSortOrder };
    this.refreshView(true);
  }

  setFocus(album) {
    this.focusedAlbum = album;
  }

  closeFocus() {
    this.focusedAlbum = null;
  }
}

export const library = new LibraryState();
