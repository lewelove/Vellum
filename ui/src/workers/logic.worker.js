import { applyFilter } from "../logic/filters.js";
import { sorters } from "../logic/sorters.js";
import { generateSidebarGroup } from "../logic/groupers.js";

// The Worker's "Source of Truth"
let rawAlbums = [];

// State for re-processing updates without requiring new payloads
let currentFilter = { key: null, val: null };
let currentSort = { key: "default" };

self.onmessage = (e) => {
  const { type, payload } = e.data;

  try {
    switch (type) {
      // 1. Ingest & Initialize
      case "INIT": {
        // Handle stringified JSON from main thread (avoids main thread parse cost)
        const parsed = typeof payload === 'string' ? JSON.parse(payload) : payload;
        
        // Handle potential envelope { type: 'INIT', data: [...] } vs raw array
        const data = Array.isArray(parsed) ? parsed : (parsed.data || []);
        
        // Data Enhancement (occurs once here)
        data.forEach(a => {
          a.title = a.ALBUM;
          a.artist = a.ALBUMARTIST;
        });

        rawAlbums = data;

        // A. Send FULL DATA to Main Thread to populate the Object Cache.
        // This is the heavy transfer (~9MB), happens only once.
        postMessage({ 
          type: "INIT_DATA", 
          data: rawAlbums,
          count: rawAlbums.length 
        });

        // B. Immediately calculate the initial view (sends IDs only).
        processView(currentFilter, currentSort);
        break;
      }

      // 2. Hot Reload / Single Update
      case "UPDATE": {
        const albumData = payload;
        // Enhance new data
        albumData.title = albumData.ALBUM;
        albumData.artist = albumData.ALBUMARTIST;

        // Update local Worker state
        const index = rawAlbums.findIndex(a => a.id === payload.id);
        if (index !== -1) rawAlbums[index] = albumData;
        else rawAlbums.push(albumData);

        // Sync this specific object to Main Thread Cache
        postMessage({ type: "UPDATE_DATA", data: albumData });
        
        // Re-run the current view
        processView(currentFilter, currentSort);
        break;
      }

      // 3. View Processing (Filter/Sort)
      case "PROCESS": {
        processView(payload.filter, payload.sort);
        break;
      }
      
      // 4. Sidebar Grouping
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

function processView(filter, sort) {
  // Update local scope state
  currentFilter = filter;
  currentSort = sort;

  const tStart = performance.now();

  // A. Filter (Operations on Full Objects)
  let result = rawAlbums;
  if (filter && filter.key) {
    result = rawAlbums.filter(a => applyFilter(a, filter.key, filter.val));
  }

  // B. Sort (Operations on Full Objects)
  // We Copy array to prevent mutation of the source
  result = [...result]; 
  const sorter = sorters[sort.key] || sorters.date_added;
  result.sort(sorter);

  // C. Optimize Output (IDs Only)
  // Map the objects to a list of ID strings.
  // Transferring Strings is O(n) but very lightweight compared to Objects.
  const viewIds = result.map(a => a.id);

  const tEnd = performance.now();

  postMessage({ 
    type: "VIEW_UPDATED", 
    ids: viewIds, 
    timing: (tEnd - tStart).toFixed(2)
  });
}
