import { connectSocket } from "./api.js";
import { player, updatePlayerState } from "./modules/player.svelte.js";
import { nav } from "./navigation.svelte.js";

class LibraryState {
  dict = $state({});
  trackPathMap = $state({});
  
  libraryViewIds = $state([]);
  shelfViewIds = $state([]);
  
  libraryAlbums = $derived(this.mapIdsToAlbums(this.libraryViewIds));
  shelfAlbums = $derived(this.mapIdsToAlbums(this.shelfViewIds));
  
  mapIdsToAlbums(ids) {
    return ids.map(id => {
      let a = this.dict[id];
      return a ? {
          id: a.id,
          title: a.ALBUM,
          artist: a.ALBUMARTIST,
          cover_hash: a.cover_hash,
          total_discs: a.total_discs,
          total_tracks: a.total_tracks,
          album_duration_time: a.album_duration_time,
          tags: a.tags
      } : null;
    }).filter(Boolean);
  }
  
  sidebarGroups = $state(new Map()); 
  isLoading = $state(true);
  isConnected = $state(false);
  focusedAlbum = $state(null);
  
  activeCollection = $state("library");
  activeFilter = $state({ key: null, val: null });
  activeSort = $state({ key: "default", order: "default" });
  userSortPreference = $state("default");
  userSortOrder = $state("default");
  activeSidebarGrouper = $state("genre");
  activeShelf = $state(null);
  
  libraryVersion = $state(0);
  shelfVersion = $state(0);

  pinnedTextures = $state(new Map());
  fullAlbumCache = $state({});
  isShaderEnabled = $state(true);
  isShaderActive = $derived(this.isShaderEnabled && player.state !== "stop");
  queuePanels = $state({ lyrics: false, tracks: true });
  themeVersion = $state(Date.now());
  
  sidebarWidth = $state(160);
  
  manifest = $state({ collections: {}, groupers: {}, sorters: {}, shelves: {} });

  config = $state({
    thumbnail_size: 200,
    shader: null
  });

  _ws = null;
  _pendingViewReset = false;

  init() {
    this._ws = connectSocket(
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
          console.error(err);
        }
      };
      reader.readAsText(event.data);
    } else {
      try {
        const json = JSON.parse(event.data);
        this.dispatchSocketAction(json);
      } catch (err) {
        console.error(err);
      }
    }
  }

  dispatchSocketAction(json) {
    if (json.type === "INIT_DICT") {
      this.dict = json.dict || {};
      this.trackPathMap = json.trackMap || {};
      if (json.manifest) this.manifest = json.manifest;
      if (json.config) this.config = { ...this.config, ...json.config };
      if (json.ui_state) this.applyPersistedState(json.ui_state);
      
      this.orchestratePrewarming();
      this.refreshView(true);
      this.refreshSidebar();
      
    } else if (json.type === "VIEW_DATA") {
      const isShelves = (nav.activeTab === "shelves");
      
      if (isShelves) {
        this.shelfViewIds = json.ids || [];
        if (this._pendingViewReset) this.shelfVersion++;
      } else {
        this.libraryViewIds = json.ids || [];
        if (this._pendingViewReset) this.libraryVersion++;
      }
      
      this.isLoading = false;
      this._pendingViewReset = false;
    } else if (json.type === "GROUP_RESULT") {
      const newMap = new Map(this.sidebarGroups);
      newMap.set(json.key, json.result);
      this.sidebarGroups = newMap;
    } else if (json.type === "MPD_STATUS") {
      updatePlayerState(json);
    } else if (json.type === "THEME_UPDATE") {
      this.themeVersion = Date.now();
    } else if (json.type === "LOGIC_UPDATE") {
      window.location.reload(); 
    } else if (json.type === "ALBUM_UPDATED") {
      if (json.dictEntry && Object.keys(json.dictEntry).length > 0) {
        this.dict[json.id] = json.dictEntry;
      } else {
        delete this.dict[json.id];
      }
      delete this.fullAlbumCache[json.id];
      if (this.focusedAlbum && this.focusedAlbum.id === json.id) {
        this.ensureFullAlbum(json.id).then(data => {
          if (data) this.focusedAlbum = data;
        });
      }
      this.orchestratePrewarming();
      this.refreshView(false);
      this.refreshSidebar();
    } else if (json.type === "CONFIG_UPDATE") {
      if (json.config) {
        this.config = { ...this.config, ...json.config };
        this.orchestratePrewarming();
      }
    }
  }

  async orchestratePrewarming() {
    const concurrencyLimit = 6;
    const queue = Object.values(this.dict);
    let pendingUpdates = false;
    let lastFlush = Date.now();

    const flush = () => {
      this.pinnedTextures = new Map(this.pinnedTextures);
      pendingUpdates = false;
      lastFlush = Date.now();
    };

    const processor = async () => {
      while (queue.length > 0) {
        const album = queue.shift();
        const url = this.getThumbnailUrl(album);
        if (!url || this.pinnedTextures.has(url)) continue;
        try {
          const res = await fetch(url);
          const blob = await res.blob();
          const bitmap = await createImageBitmap(blob, {
            premultiplyAlpha: 'none',
            colorSpaceConversion: 'default'
          });
          this.pinnedTextures.set(url, bitmap);
          pendingUpdates = true;
          if (Date.now() - lastFlush > 100) flush();
        } catch (err) {}
      }
      if (pendingUpdates) flush();
    };

    const workers = Array.from({ length: concurrencyLimit }, () => processor());
    await Promise.all(workers);
    if (pendingUpdates) flush();
  }

  applyPersistedState(state) {
      nav.activeTab = state.activeTab || "home";
      this.activeCollection = state.activeCollection || "library";
      this.userSortPreference = state.sortKey || "default";
      this.userSortOrder = state.sortOrder || "default";
      this.activeSort = { key: this.userSortPreference, order: this.userSortOrder };
      this.activeSidebarGrouper = state.groupKey || "genre";
      this.activeFilter = state.filter || { key: null, val: null };
      this.activeShelf = state.activeShelf || null;
      this.isShaderEnabled = state.isShaderEnabled ?? true;
      this.queuePanels = state.queuePanels || { lyrics: false, tracks: true };
      this.sidebarWidth = state.sidebarWidth || 160;
  }

  persistState() {
      fetch("/api/state", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
              activeTab: nav.activeTab,
              activeCollection: this.activeCollection,
              sortKey: this.userSortPreference,
              sortOrder: this.userSortOrder,
              groupKey: this.activeSidebarGrouper,
              filter: $state.snapshot(this.activeFilter),
              activeShelf: this.activeShelf,
              isShaderEnabled: this.isShaderEnabled,
              queuePanels: $state.snapshot(this.queuePanels),
              sidebarWidth: this.sidebarWidth
          })
      }).catch(err => console.error(err));
  }

  refreshView(resetScroll = true) {
    if (!this._ws || this._ws.readyState !== WebSocket.OPEN) return;
    this._pendingViewReset = resetScroll;
    
    if (nav.activeTab === "shelves") {
        this._ws.send(JSON.stringify({
            type: "SHELF_REQUEST",
            shelf: this.activeShelf || Object.keys(this.availableShelves)[0]
        }));
    } else {
        this._ws.send(JSON.stringify({
            type: "VIEW_REQUEST",
            collection: this.activeCollection,
            sort: this.activeSort.key,
            reverse: this.activeSort.order === "reverse",
            filter: this.activeFilter
        }));
    }
  }

  refreshSidebar() {
    if (!this._ws || this._ws.readyState !== WebSocket.OPEN) return;
    this._ws.send(JSON.stringify({
        type: "GROUP_REQUEST",
        collection: this.activeCollection,
        key: this.activeSidebarGrouper
    }));
  }

  getSidebarGroup(key) {
    if (!this.sidebarGroups.has(key) && this._ws?.readyState === WebSocket.OPEN) {
        this.refreshSidebar();
        return [];
    }
    return this.sidebarGroups.get(key) || [];
  }

  getTrackByPath(path) {
    return this.trackPathMap[path];
  }

  getThumbnailUrl(album) {
    if (!album || !album.cover_hash) return "";
    const size = this.config.thumbnail_size || 200;
    return `/api/covers/${size}px/${album.cover_hash}`;
  }

  getAlbumCoverUrl(albumId) {
    const album = this.dict[albumId];
    if (!album || !album.cover_hash) return "";
    return `/api/assets/cover/${encodeURIComponent(albumId)}?v=${album.cover_hash}`;
  }

  get availableCollections() { return this.manifest.collections || {}; }
  get availableFacets() { return this.manifest.groupers || {}; }
  get availableSorters() { return this.manifest.sorters || {}; }
  get availableShelves() { return this.manifest.shelves || {}; }

  get visibleFacets() {
    const collection = this.availableCollections[this.activeCollection];
    if (collection && collection.allowed_groupers) {
      const res = {};
      for (const k of collection.allowed_groupers) {
        if (this.availableFacets[k]) res[k] = this.availableFacets[k].label || k;
      }
      return res;
    }
    const res = {};
    for (const [k, v] of Object.entries(this.availableFacets)) res[k] = v.label || k;
    return res;
  }

  get visibleSorters() {
    const collection = this.availableCollections[this.activeCollection];
    if (collection && collection.allowed_sorters) {
      const res = {};
      for (const k of collection.allowed_sorters) {
        if (this.availableSorters[k]) res[k] = this.availableSorters[k].label || k;
      }
      return res;
    }
    const res = {};
    for (const [k, v] of Object.entries(this.availableSorters)) res[k] = v.label || k;
    return res;
  }

  setCollection(key) {
    this.activeCollection = key;
    this.activeFilter = { key: null, val: null };
    this.focusedAlbum = null;
    const collection = this.availableCollections[key];
    if (collection) {
        if (collection.allowed_groupers && !collection.allowed_groupers.includes(this.activeSidebarGrouper)) {
            this.activeSidebarGrouper = collection.allowed_groupers[0] || Object.keys(this.availableFacets)[0] || "genre";
        }
        if (collection.allowed_sorters && !collection.allowed_sorters.includes(this.userSortPreference)) {
            this.userSortPreference = collection.allowed_sorters[0] || Object.keys(this.availableSorters)[0] || "default";
            this.activeSort = { key: this.userSortPreference, order: this.userSortOrder };
        }
    }
    this.refreshView(true);
    this.refreshSidebar();
    this.persistState();
  }

  setShelf(key) {
    this.activeShelf = key;
    this.focusedAlbum = null;
    this.refreshView(true);
    this.persistState();
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

  async ensureFullAlbum(id) {
    if (!id) return null;
    if (this.fullAlbumCache[id]) return this.fullAlbumCache[id];
    try {
        const res = await fetch(`/api/album/${encodeURIComponent(id)}`);
        if (res.ok) {
            const data = await res.json();
            data.id = id;
            this.fullAlbumCache[id] = data;
            return data;
        }
    } catch (err) {
        console.error(err);
    }
    return null;
  }

  async setFocus(album) {
    this.focusedAlbum = await this.ensureFullAlbum(album.id);
  }

  closeFocus() {
    this.focusedAlbum = null;
  }
  
  toggleShader() {
    this.isShaderEnabled = !this.isShaderEnabled;
    this.persistState();
  }

  toggleQueuePanel(key) {
    this.queuePanels[key] = !this.queuePanels[key];
    this.persistState();
  }
}

export const library = new LibraryState();
