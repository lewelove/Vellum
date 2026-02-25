import { applyFilter } from "../logic/filters.js";
import { sorters } from "../logic/sorters.js";
import { generateSidebarGroup } from "../logic/groupers.js";

let rawAlbums = [];

let currentFilter = { key: null, val: null };
let currentSort = { key: "default", order: "default" };

self.onmessage = (e) => {
  const { type, payload } = e.data;

  try {
    switch (type) {

      case "INIT": {
        let data = [];
        const sourceData = payload.data || payload;

        if (Array.isArray(sourceData)) {
          data = sourceData;
        } else if (sourceData && Array.isArray(sourceData.data)) {
          data = sourceData.data;
        }
        
        // Normalize nested and lowercase data for the UI
        data.forEach(a => {
          // 1. Flatten the 'info' object to the top level
          if (a.info) {
            Object.assign(a, a.info);
          }
          
          // 2. Map lowercase keys to the uppercase keys expected by logic/filters/sorters
          a.ALBUM = a.ALBUM || a.album;
          a.ALBUMARTIST = a.ALBUMARTIST || a.albumartist;
          a.GENRE = a.GENRE || a.genre;
          a.DATE = a.DATE || a.date;
          a.COMMENT = a.COMMENT || a.comment;
          a.CUSTOM_ALBUMARTIST = a.CUSTOM_ALBUMARTIST || a.custom_albumartist;

          // 3. Set display helpers
          a.title = a.ALBUM;
          a.artist = a.ALBUMARTIST;
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

      case "UPDATE": {
        const albumData = payload;
        
        // Normalize update payload
        if (albumData.info) Object.assign(albumData, albumData.info);
        albumData.ALBUM = albumData.ALBUM || albumData.album;
        albumData.ALBUMARTIST = albumData.ALBUMARTIST || albumData.albumartist;
        albumData.title = albumData.ALBUM;
        albumData.artist = albumData.ALBUMARTIST;

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

  currentFilter = filter;
  currentSort = sort;

  const tStart = performance.now();

  let result = rawAlbums;
  if (filter && filter.key) {
    result = rawAlbums.filter(a => applyFilter(a, filter.key, filter.val));
  }

  result = [...result]; 
  const sorter = sorters[sort.key] || sorters.default;
  result.sort(sorter);

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
