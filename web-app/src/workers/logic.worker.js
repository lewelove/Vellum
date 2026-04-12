import * as duckdb from '@duckdb/duckdb-wasm';
import duckdb_wasm from '@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm?url';
import mvp_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-mvp.worker.js?url';
import duckdb_wasm_eh from '@duckdb/duckdb-wasm/dist/duckdb-eh.wasm?url';
import eh_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-eh.worker.js?url';

import { coreFacets } from "../logic/core_facets.js";
import { coreSorters } from "../logic/core_sorters.js";
import { coreShelves } from "../logic/core_shelves.js";

let db = null;
let conn = null;
let dbInitPromise = null;

let currentShelfKey = "library";
let currentFilter = { key: null, val: null };
let currentSort = { key: "default", order: "default" };

let registryShelves = {};
let registryFacets = {};
let registrySorters = {};

async function initDuckDB() {
  try {
    const MANUAL_BUNDLES = {
      mvp: { mainModule: duckdb_wasm, mainWorker: mvp_worker },
      eh: { mainModule: duckdb_wasm_eh, mainWorker: eh_worker },
    };
    
    const bundle = await duckdb.selectBundle(MANUAL_BUNDLES);
    const worker = new Worker(bundle.mainWorker);
    const logger = new duckdb.VoidLogger();
    
    db = new duckdb.AsyncDuckDB(logger, worker);
    await db.instantiate(bundle.mainModule, bundle.pthreadWorker);
    conn = await db.connect();
  } catch (e) {
    console.error("Failed to initialize DuckDB:", e);
  }
}

dbInitPromise = initDuckDB();

async function loadRegistries() {
  registryShelves = { ...coreShelves };
  registryFacets = { ...coreFacets };
  registrySorters = { ...coreSorters };

  try {
    const userShelves = await import(/* @vite-ignore */ `/api/theme/shelves.js?v=${Date.now()}`);
    if (userShelves.shelves) Object.assign(registryShelves, userShelves.shelves);
  } catch (e) {}

  try {
    const userFacets = await import(/* @vite-ignore */ `/api/theme/facets.js?v=${Date.now()}`);
    if (userFacets.facets) Object.assign(registryFacets, userFacets.facets);
  } catch (e) {}

  try {
    const userSorters = await import(/* @vite-ignore */ `/api/theme/sorters.js?v=${Date.now()}`);
    if (userSorters.sorters) Object.assign(registrySorters, userSorters.sorters);
  } catch (e) {}

  const availableShelves = {};
  for (const [k, v] of Object.entries(registryShelves)) {
    availableShelves[k] = { label: v.label || k, facets: v.facets, sorters: v.sorters };
  }

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
    shelves: availableShelves,
    facets: availableFacets, 
    sorters: availableSorters 
  });
}

self.onmessage = async (e) => {
  await dbInitPromise;
  const { type, payload } = e.data;

  try {
    switch (type) {
      case "INIT": {
        await loadRegistries();

        let data =[];
        const sourceData = payload.data || payload;

        if (Array.isArray(sourceData)) {
          data = sourceData;
        } else if (sourceData && Array.isArray(sourceData.data)) {
          data = sourceData.data;
        }
        
        data.forEach(a => {
          if (a.info) Object.assign(a, a.info);
          a.title = a.ALBUM;
          a.artist = a.ALBUMARTIST;
          if (a.tracks) {
            a.tracks.forEach(t => {
              if (t.info) Object.assign(t, t.info);
              t.albumId = a.id;
            });
          }
        });

        // Initialize natively typed JSON schema
        await conn.query(`CREATE TABLE IF NOT EXISTS library (id VARCHAR PRIMARY KEY, data JSON)`);
        await conn.query(`DELETE FROM library`);

        if (data.length > 0) {
          // NDJSON is much faster for DuckDB to ingest than a JSON array
          const jsonLines = data.map(a => JSON.stringify(a)).join('\n');
          await db.registerFileText('library.json', jsonLines);
          // CAST to native JSON type enforces binary tree representation instead of VARCHAR
          await conn.query(`INSERT INTO library SELECT json->>'$.id', CAST(json AS JSON) FROM read_json_objects('library.json', format='newline_delimited')`);
        }

        let initialShelf = "library";
        if (payload.ui_state) {
          if (payload.ui_state.activeShelf) initialShelf = payload.ui_state.activeShelf;
          if (payload.ui_state.filter) currentFilter = payload.ui_state.filter;
          if (payload.ui_state.sortKey) {
            currentSort = { 
              key: payload.ui_state.sortKey,
              order: payload.ui_state.sortOrder || "default"
            };
          }
        }

        postMessage({ type: "INIT_DATA", data, count: data.length });
        await processView(initialShelf, currentFilter, currentSort);
        break;
      }

      case "RELOAD_LOGIC": {
        await loadRegistries();
        await processView(currentShelfKey, currentFilter, currentSort);
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

        const jsonLines = JSON.stringify(albumData);
        await db.registerFileText('update.json', jsonLines);
        await conn.query(`DELETE FROM library WHERE id = '${albumData.id.replace(/'/g, "''")}'`);
        await conn.query(`INSERT INTO library SELECT json->>'$.id', CAST(json AS JSON) FROM read_json_objects('update.json', format='newline_delimited')`);

        postMessage({ type: "UPDATE_DATA", data: albumData });
        await processView(currentShelfKey, currentFilter, currentSort);
        break;
      }

      case "PROCESS": {
        await processView(payload.shelf, payload.filter, payload.sort);
        break;
      }
      
      case "GROUP": {
        await handleGroup(payload.key);
        break;
      }
    }
  } catch (err) {
    console.error("Worker Logic Error:", err);
  }
};

async function processView(shelf, filter, sort) {
  currentShelfKey = shelf || "library";
  currentFilter = filter;
  currentSort = sort;

  const tStart = performance.now();

  const shelfDef = registryShelves[currentShelfKey];
  const shelfWhere = shelfDef ? shelfDef.where : "1=1";

  let filterWhere = "1=1";
  if (currentFilter && currentFilter.key === 'search') {
    const q = currentFilter.val.toLowerCase().replace(/'/g, "''");
    // Text search on tracks is much faster if we don't parse the JSON array at runtime
    filterWhere = `(LOWER(data->>'$.ALBUM') LIKE '%${q}%' OR LOWER(data->>'$.ALBUMARTIST') LIKE '%${q}%' OR LOWER(data->>'$.tracks') LIKE '%${q}%')`;
  } else if (currentFilter && currentFilter.key) {
    const facet = registryFacets[currentFilter.key];
    if (facet && facet.filterWhere) {
      filterWhere = facet.filterWhere(currentFilter.val);
    }
  }

  const sorterObj = registrySorters[currentSort.key] || registrySorters.default;
  let orderBy = sorterObj ? sorterObj.orderBy : "id ASC";
  
  if (currentSort.order === "reverse") {
    orderBy = orderBy.replace(/\bASC\b/g, '_TMP_').replace(/\bDESC\b/g, 'ASC').replace(/\b_TMP_\b/g, 'DESC');
  }

  const query = `SELECT id FROM library WHERE (${shelfWhere}) AND (${filterWhere}) ORDER BY ${orderBy}`;
  
  try {
    const arrowResult = await conn.query(query);
    
    // BLAZING FAST ARROW EXTRACTION 
    // Extracts vector directly without allocating JS Objects via .toArray()
    const idVector = arrowResult.getChild("id");
    const viewIds = idVector ? Array.from(idVector).map(id => id.toString()) :[];
    
    const tEnd = performance.now();

    postMessage({ 
      type: "VIEW_UPDATED", 
      ids: viewIds, 
      timing: (tEnd - tStart).toFixed(2)
    });
  } catch (err) {
    console.error("DuckDB Query Error:", err);
  }
}

async function handleGroup(key) {
  const shelfDef = registryShelves[currentShelfKey];
  const shelfWhere = shelfDef ? shelfDef.where : "1=1";
  const facet = registryFacets[key];

  if (facet) {
    const query = `
      SELECT ${facet.select} AS value, COUNT(*) as count 
      FROM library 
      WHERE (${shelfWhere}) 
      GROUP BY value 
      HAVING value IS NOT NULL
      ORDER BY ${facet.orderBy || 'count DESC'}
    `;
    
    try {
      const arrowResult = await conn.query(query);
      // toArray() is safe here because grouped results are extremely small (e.g. 50 genres)
      let result = arrowResult.toArray().map(row => ({
        label: facet.getLabel ? facet.getLabel(row.value) : (row.value ? row.value.toString() : "Unknown"),
        value: row.value ? row.value.toString() : "Unknown",
        count: Number(row.count),
        filterTarget: key
      }));
      postMessage({ type: "GROUP_RESULT", key: key, result });
    } catch (err) {
      console.error("DuckDB Group Error:", err);
    }
  }
}
