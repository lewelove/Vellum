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
        
        data.forEach(a => {
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
