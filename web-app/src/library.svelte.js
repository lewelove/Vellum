import { connectSocket } from "./api.js";
import { player, updatePlayerState } from "./modules/player.svelte.js";
import { nav } from "./navigation.svelte.js";
import LogicWorker from "./workers/logic.worker.js?worker"; 
import { theme } from "./theme.svelte.js";

class LibraryState {

  albums = $state([]); 
  sidebarGroups = $state(new Map()); 
  isLoading = $state(true);
  isConnected = $state(false);
  focusedAlbum = $state(null);
  activeShelf = $state("library");
  activeFilter = $state({ key: null, val: null });
  activeSort = $state({ key: "default", order: "default" });
  userSortPreference = $state("default");
  userSortOrder = $state("default");
  activeSidebarGrouper = $state("genre");
  viewVersion = $state(0);
  albumCache = $state(new Map());
  trackPathMap = $state(new Map());
  pinnedTextures = $state(new Map());
  pinnedTextTextures = $state(new Map());
  isShaderEnabled = $state(true);
  queuePanels = $state({ lyrics: false, tracks: true });
  themeVersion = $state(Date.now());
  
  availableShelves = $state({});
  availableFacets = $state({});
  availableSorters = $state({});

  config = $state({
    thumbnail_size: 200,
    shader: null
  });

  worker = null;
  _tRequest = 0;
  _pendingViewReset = false;

  init() {
    this.worker = new LogicWorker();
    
    this.worker.onmessage = (e) => {
      const { type, data, ids, result, key, facets, sorters, shelves } = e.data;

      if (type === "LOGIC_LOADED") {
        this.availableShelves = shelves;
        this.availableFacets = facets;
        this.availableSorters = sorters;
        this.refreshSidebar();
      }

      else if (type === "INIT_DATA") {
        const newTrackMap = new Map();
        const newAlbumCache = new Map();

        data.forEach(a => {
          newAlbumCache.set(a.id, a);
          if (a.tracks) {
            a.tracks.forEach(t => {
              if (t.track_library_path) {
                newTrackMap.set(t.track_library_path, t);
              }
            });
          }
        });

        this.trackPathMap = newTrackMap;
        this.albumCache = newAlbumCache;

        this.orchestratePrewarming(data);
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
        if (this.focusedAlbum?.id === data.id) {
          this.focusedAlbum = data;
        }
      }

      else if (type === "VIEW_UPDATED") {
        this.albums = ids.map(id => this.albumCache.get(id)).filter(Boolean);
        this.isLoading = false;

        if (this._pendingViewReset) {
            this.viewVersion++;
            this._pendingViewReset = false;
        }
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
    if (json.type === "UPDATE") {
      this.worker.postMessage({ type: "UPDATE", payload: json.payload });
    } else if (json.type === "INIT") {
      if (json.config) {
        this.config = { ...this.config, ...json.config };
      }
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
    } else if (json.type === "CONFIG_UPDATE") {
      if (json.config) {
        this.config = { ...this.config, ...json.config };
      }
    } else if (json.type === "MPD_STATUS") {
      updatePlayerState(json);
    } else if (json.type === "THEME_UPDATE") {
      this.themeVersion = Date.now();
    } else if (json.type === "LOGIC_UPDATE") {
      this.worker.postMessage({ type: "RELOAD_LOGIC" });
    }
  }

  async generateTextBitmap(album) {
    if (!album) return null;
    
    const coverSize = theme.albumGrid["cover-size"] || 190;
    const lhTitle = theme.albumGrid["font-line-height-title"] || 16;
    const gapLesser = theme.albumGrid["text-gap-lesser"] || 2;
    const lhArtist = theme.albumGrid["font-line-height-artist"] || 14;
    const textBlockHeight = lhTitle + gapLesser + lhArtist;

    const dpr = window.devicePixelRatio || 1;
    const w = coverSize;
    const h = textBlockHeight;

    if (w <= 0 || h <= 0) return null;

    const canvas = new OffscreenCanvas(w * dpr, (h + 2) * dpr);
    const ctx = canvas.getContext('2d', { alpha: false });
    
    ctx.scale(dpr, dpr);
    ctx.translate(0, 1);
    
    ctx.fillStyle = "#333333";
    ctx.fillRect(0, -1, w, h + 2);
    
    ctx.shadowColor = "rgba(0, 0, 0, 0.1)";
    ctx.shadowBlur = 4;
    ctx.shadowOffsetX = 0;
    ctx.shadowOffsetY = 0;

    const fontStack = "Inter Vellum, 'Noto Sans', system-ui, sans-serif";
    
    const cTitle = theme.palette[theme.colors["text-main"]] || "#ffffff";
    const sTitle = theme.typography["font-size-title"] || 14;
    const wTitle = theme.typography["font-weight-title"] || 400;

    ctx.fillStyle = cTitle;
    ctx.font = `${wTitle} ${sTitle}px ${fontStack}`;
    ctx.textBaseline = "middle"; 
    
    const fitText = (ctx, text, maxWidth) => {
      if (!text) return "";
      let ellipsis = "...";
      let width = ctx.measureText(text).width;
      if (width <= maxWidth) return text;
      
      let len = text.length;
      while (width > maxWidth && len > 0) {
        len--;
        width = ctx.measureText(text.substring(0, len) + ellipsis).width;
      }
      return text.substring(0, len) + ellipsis;
    };

    const titleY = lhTitle / 2;
    ctx.fillText(fitText(ctx, album.title, w), 0, titleY);
    
    const cArtist = theme.palette[theme.colors["text-muted"]] || "#cccccc";
    const sArtist = theme.typography["font-size-artist"] || 12;
    const wArtist = theme.typography["font-weight-artist"] || 400;

    ctx.fillStyle = cArtist;
    ctx.font = `${wArtist} ${sArtist}px ${fontStack}`;
    
    const artistY = lhTitle + gapLesser + (lhArtist / 2);
    ctx.fillText(fitText(ctx, album.artist, w), 0, artistY);

    return canvas.transferToImageBitmap();
  }

  async orchestratePrewarming(albumData) {
    await document.fonts.ready;

    const concurrencyLimit = 6;
    const queue = [...albumData];
    let pendingUpdates = false;
    let lastFlush = Date.now();

    const flush = () => {
      this.pinnedTextures = new Map(this.pinnedTextures);
      this.pinnedTextTextures = new Map(this.pinnedTextTextures);
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

          if (!this.pinnedTextTextures.has(album.id)) {
              const textBitmap = await this.generateTextBitmap(album);
              if (textBitmap) {
                  this.pinnedTextTextures.set(album.id, textBitmap);
              }
          }

          pendingUpdates = true;

          if (Date.now() - lastFlush > 100) {
            flush();
          }
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
      this.activeShelf = state.activeShelf || "library";
      this.userSortPreference = state.sortKey || "default";
      this.userSortOrder = state.sortOrder || "default";
      this.activeSort = { key: this.userSortPreference, order: this.userSortOrder };
      this.activeSidebarGrouper = state.groupKey || "genre";
      this.activeFilter = state.filter || { key: null, val: null };
      this.isShaderEnabled = state.isShaderEnabled ?? true;
      this.queuePanels = state.queuePanels || { lyrics: false, tracks: true };
  }

  persistState() {
      fetch("/api/state", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
              activeTab: nav.activeTab,
              activeShelf: this.activeShelf,
              sortKey: this.userSortPreference,
              sortOrder: this.userSortOrder,
              groupKey: this.activeSidebarGrouper,
              filter: $state.snapshot(this.activeFilter),
              isShaderEnabled: this.isShaderEnabled,
              queuePanels: $state.snapshot(this.queuePanels)
          })
      }).catch(err => console.error(err));
  }

  refreshView(resetScroll = true) {
    if (!this.worker) return;
    this._pendingViewReset = resetScroll;
    
    this.worker.postMessage({
      type: "PROCESS",
      payload: {
        shelf: this.activeShelf,
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
        return[];
    }
    return this.sidebarGroups.get(key) ||[];
  }

  getTrackByPath(path) {
    return this.trackPathMap.get(path);
  }

  getThumbnailUrl(album) {
    if (!album || !album.cover_hash) return "";
    const size = this.config.thumbnail_size || 200;
    return `/api/covers/${size}px/${album.cover_hash}`;
  }

  getAlbumCoverUrl(albumId) {
    const album = this.albumCache.get(albumId);
    if (!album) return "";
    const cp = album.cover_path;
    const ch = album.cover_hash;
    if (!cp || cp === "default_cover.png") {
      return "";
    }
    return `/api/assets/cover/${encodeURIComponent(album.id)}?v=${ch}`;
  }

  get visibleFacets() {
    const shelf = this.availableShelves[this.activeShelf];
    if (shelf && shelf.facets) {
      const res = {};
      for (const k of shelf.facets) {
        if (this.availableFacets[k]) res[k] = this.availableFacets[k];
      }
      return res;
    }
    return this.availableFacets;
  }

  get visibleSorters() {
    const shelf = this.availableShelves[this.activeShelf];
    if (shelf && shelf.sorters) {
      const res = {};
      for (const k of shelf.sorters) {
        if (this.availableSorters[k]) res[k] = this.availableSorters[k];
      }
      return res;
    }
    return this.availableSorters;
  }

  setShelf(key) {
    this.activeShelf = key;
    this.activeFilter = { key: null, val: null };
    this.focusedAlbum = null;

    const shelf = this.availableShelves[key];
    if (shelf) {
        if (shelf.facets && !shelf.facets.includes(this.activeSidebarGrouper)) {
            this.activeSidebarGrouper = shelf.facets[0] || Object.keys(this.availableFacets)[0] || "genre";
        }
        if (shelf.sorters && !shelf.sorters.includes(this.userSortPreference)) {
            this.userSortPreference = shelf.sorters[0] || Object.keys(this.availableSorters)[0] || "default";
            this.activeSort = { key: this.userSortPreference, order: this.userSortOrder };
        }
    }

    this.refreshView(true);
    this.refreshSidebar();
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

  setFocus(album) {
    this.focusedAlbum = album;
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
