import { coreFacets } from "../logic/core_facets.js";
import { coreSorters } from "../logic/core_sorters.js";

let rawAlbums = [];

let currentFilter = { key: null, val: null };
let currentSort = { key: "default", order: "default" };

let registryFacets = {};
let registrySorters = {};

async function loadRegistries() {
  registryFacets = { ...coreFacets };
  registrySorters = { ...coreSorters };

  try {
    const userFacets = await import(/* @vite-ignore */ `/api/theme/facets.js?v=${Date.now()}`);
    if (userFacets.facets) {
      Object.assign(registryFacets, userFacets.facets);
    }
  } catch (e) {}

  try {
    const userSorters = await import(/* @vite-ignore */ `/api/theme/sorters.js?v=${Date.now()}`);
    if (userSorters.sorters) {
      Object.assign(registrySorters, userSorters.sorters);
    }
  } catch (e) {}

  const availableFacets = {};
  for (const [k, v] of Object.entries(registryFacets)) {
    availableFacets[k] = v.label || k;
  }

  const availableSorters = {};
  for (const [k, v] of Object.entries(registrySorters)) {
    availableSorters[k] = v.label || k;
  }

  postMessage({ 
    type: "LOGIC_LOADED", 
    facets: availableFacets, 
    sorters: availableSorters 
  });
}

function generateBuckets(albums, facetKey) {
  const facet = registryFacets[facetKey];
  if (!facet) return [];

  const map = new Map();
  
  albums.forEach(album => {
    const raw = facet.getValue(album);
    if (raw === null || raw === undefined) return;
    
    const vals = Array.isArray(raw) ? raw : [raw];
    vals.forEach(v => {
      if (!map.has(v)) {
        const label = facet.getLabel ? facet.getLabel(v) : v;
        map.set(v, { label, value: v, count: 0, filterTarget: facetKey });
      }
      map.get(v).count++;
    });
  });

  if (facet.sortBuckets) {
    return facet.sortBuckets(map).map(kv => kv[1]);
  }

  return Array.from(map.values()).sort((a, b) => 
    String(a.value).localeCompare(String(b.value), undefined, { numeric: true })
  );
}

function isMatch(album, facetKey, filterValue) {
  const facet = registryFacets[facetKey];
  if (!facet) return true;
  
  if (facet.filter) return facet.filter(album, filterValue);

  const val = facet.getValue(album);
  if (val === null || val === undefined) return false;
  
  if (Array.isArray(val)) return val.includes(filterValue);
  return val === filterValue;
}

self.onmessage = async (e) => {
  const { type, payload } = e.data;

  try {
    switch (type) {
      case "INIT": {
        await loadRegistries();

        let data = [];
        const sourceData = payload.data || payload;

        if (Array.isArray(sourceData)) {
          data = sourceData;
        } else if (sourceData && Array.isArray(sourceData.data)) {
          data = sourceData.data;
        }
        
        data.forEach(a => {
          if (a.info) {
            Object.assign(a, a.info);
          }
          a.title = a.ALBUM;
          a.artist = a.ALBUMARTIST;
          if (a.tracks) {
            a.tracks.forEach(t => {
                if (t.info) Object.assign(t, t.info);
                t.albumId = a.id;
            });
          }
        });

        rawAlbums = data;

        if (payload.ui_state) {
          if (payload.ui_state.filter) {
            currentFilter = payload.ui_state.filter;
          }
          if (payload.ui_state.sortKey) {
            currentSort = { 
              key: payload.ui_state.sortKey,
              order: payload.ui_state.sortOrder || "default"
            };
          }
        }

        postMessage({ 
          type: "INIT_DATA", 
          data: rawAlbums,
          count: rawAlbums.length 
        });

        processView(currentFilter, currentSort);
        break;
      }

      case "RELOAD_LOGIC": {
        await loadRegistries();
        processView(currentFilter, currentSort);
        break;
      }

      case "UPDATE": {
        const albumData = payload;
        
        if (albumData.info) Object.assign(albumData, albumData.info);
        albumData.title = albumData.ALBUM;
        albumData.artist = albumData.ALBUMARTIST;

        if (albumData.tracks) {
            albumData.tracks.forEach(t => {
                if (t.info) Object.assign(t, t.info);
                t.albumId = albumData.id;
            });
        }

        const index = rawAlbums.findIndex(a => a.id === payload.id);
        if (index !== -1) {
          rawAlbums[index] = albumData;
        } else {
          rawAlbums.push(albumData);
        }

        postMessage({ type: "UPDATE_DATA", data: albumData });
        processView(currentFilter, currentSort);
        break;
      }

      case "PROCESS": {
        processView(payload.filter, payload.sort);
        break;
      }
      
      case "GROUP": {
        const result = generateBuckets(rawAlbums, payload.key);
        postMessage({ type: "GROUP_RESULT", key: payload.key, result });
        break;
      }
    }
  } catch (err) {
    console.error("Worker Logic Error:", err);
  }
};

function processView(filter, sort) {
  currentFilter = filter;
  currentSort = sort;

  const tStart = performance.now();

  let result = rawAlbums;
  
  if (filter && filter.key === 'search') {
    const q = filter.val.toLowerCase();
    result = rawAlbums.filter(a => {
      if ((a.ALBUM && a.ALBUM.toLowerCase().includes(q)) || (a.ALBUMARTIST && a.ALBUMARTIST.toLowerCase().includes(q))) return true;
      if (a.tracks && a.tracks.some(t => t.TITLE && t.TITLE.toLowerCase().includes(q))) return true;
      return false;
    });
  } else if (filter && filter.key) {
    result = rawAlbums.filter(a => isMatch(a, filter.key, filter.val));
  }

  result = [...result]; 
  
  const sorterObj = registrySorters[sort.key] || registrySorters.default;
  const sorterFn = sorterObj ? sorterObj.sort : () => 0;
  result.sort(sorterFn);

  if (sort.order === "reverse") {
    result.reverse();
  }

  const viewIds = result.map(a => a.id);
  const tEnd = performance.now();

  postMessage({ 
    type: "VIEW_UPDATED", 
    ids: viewIds, 
    timing: (tEnd - tStart).toFixed(2)
  });
}
