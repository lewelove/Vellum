import { applyFilter } from "../logic/filters.js";
import { sorters } from "../logic/sorters.js";
import { generateSidebarGroup } from "../logic/groupers.js";

let rawAlbums = [];

self.onmessage = (e) => {
  const { type, payload } = e.data;

  try {
    switch (type) {
      // 1. Ingest (Parse off-thread)
      case "INIT": {
        // payload is the raw JSON string or object
        const parsed = typeof payload === 'string' ? JSON.parse(payload) : payload;
        
        // FIX: Extract array from envelope { type: 'INIT', data: [...] } if present
        const data = Array.isArray(parsed) ? parsed : (parsed.data || []);
        
        // Enhance data (moved from library.svelte.js)
        data.forEach(a => {
          a.title = a.ALBUM;
          a.artist = a.ALBUMARTIST;
        });

        rawAlbums = data;
        
        // Return stats + initial view
        postMessage({ 
          type: "READY", 
          count: rawAlbums.length,
          view: rawAlbums // Initial view is unsorted/unfiltered
        });
        break;
      }

      // 2. Incremental Updates (Hot Reload)
      case "UPDATE": {
        // payload is the single album object
        const albumData = payload;
        albumData.title = albumData.ALBUM;
        albumData.artist = albumData.ALBUMARTIST;

        const index = rawAlbums.findIndex(a => a.id === payload.id);
        if (index !== -1) rawAlbums[index] = albumData;
        else rawAlbums.push(albumData);

        // Re-run the current view logic immediately
        processView(currentFilter, currentSort);
        break;
      }

      // 3. The Heavy Lifting
      case "PROCESS": {
        const { filter, sort } = payload;
        processView(filter, sort);
        break;
      }
      
      // 4. Sidebar Counts
      case "GROUP": {
        const result = generateSidebarGroup(rawAlbums, payload.key);
        postMessage({ type: "GROUP_RESULT", key: payload.key, result });
        break;
      }
    }
  } catch (err) {
    console.error("Worker Logic Error:", err);
  }
};

// State for re-processing updates
let currentFilter = { key: null, val: null };
let currentSort = { key: "default" };

function processView(filter, sort) {
  // Update local state
  currentFilter = filter;
  currentSort = sort;

  const tStart = performance.now();

  // A. Filter
  let result = rawAlbums;
  if (filter && filter.key) {
    result = rawAlbums.filter(a => applyFilter(a, filter.key, filter.val));
  }

  // B. Sort
  // We copy the array because .sort() mutates in place
  result = [...result]; 
  const sorter = sorters[sort.key] || sorters.date_added;
  result.sort(sorter);

  const tEnd = performance.now();

  // C. Respond
  postMessage({ 
    type: "VIEW_UPDATED", 
    data: result,
    timing: (tEnd - tStart).toFixed(2)
  });
}
