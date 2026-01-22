import * as duckdb from '@duckdb/duckdb-wasm';
import { ConsoleLogger } from '@duckdb/duckdb-wasm';

// We define the bundles relative to the public/static folder or CDN
// In a Vite/Webpack setup, these resolve to the actual WASM files
const JSDELIVR_BUNDLES = duckdb.getJsDelivrBundles();

let db = null;
let conn = null;

const init = async () => {
    if (db) return;

    // 1. Select the best bundle for the browser (MVP vs EH vs COI)
    const bundle = await duckdb.selectBundle(JSDELIVR_BUNDLES);
    
    // 2. Instantiate the worker and logger
    const worker = await duckdb.createWorker(bundle.mainWorker);
    const logger = new ConsoleLogger();

    // 3. Instantiate the Async Database
    db = new duckdb.AsyncDuckDB(logger, worker);
    await db.instantiate(bundle.mainModule, bundle.pthreadWorker);

    // 4. Create a persistent connection
    conn = await db.connect();
    
    postMessage({ type: 'READY' });
};

const loadJSON = async (jsonString) => {
    if (!db || !conn) await init();

    // 1. Register the raw JSON text into the Virtual File System (VFS)
    // We name it 'import.json'. This is instant (in-memory pointer).
    await db.registerFileText('import.json', jsonString);

    // 2. Schema-less Ingestion
    // read_json_auto detects keys, types, and nested arrays (tracks) automatically.
    // We replace the 'library' table entirely to ensure perfect sync with the patch.
    await conn.query(`
        CREATE OR REPLACE TABLE library AS 
        SELECT * FROM read_json_auto('import.json')
    `);

    // Clean up VFS to save memory
    await db.registerFileText('import.json', '');
    
    postMessage({ type: 'UPDATED' });
};

const runQuery = async ({ filter, sort, limit = 50, offset = 0 }) => {
    if (!conn) return;

    // Base query selects everything. 
    // Since DuckDB inferred the schema, 'tracks' is just a column of type LIST.
    let sql = `SELECT * FROM library`;
    
    // 1. Dynamic Filtering
    // We assume 'filter' is an object like { genre: 'Rock', year: 2020 }
    // This allows filtering by any album-scope tag present in the JSON.
    const clauses = [];
    if (filter) {
        Object.entries(filter).forEach(([key, value]) => {
            // Basic sanitization to prevent SQL injection in column names
            // In production, validate 'key' against known schema columns
            if (typeof value === 'string') {
                // Case-insensitive search
                clauses.push(`${key} ILIKE '%${value}%'`);
            } else {
                clauses.push(`${key} = ${value}`);
            }
        });
    }

    if (clauses.length > 0) {
        sql += ` WHERE ${clauses.join(' AND ')}`;
    }

    // 2. Sorting
    // We sort by album tags. 
    if (sort && sort.column) {
        sql += ` ORDER BY ${sort.column} ${sort.direction || 'ASC'}`;
    }

    // 3. Pagination (Virtualization Support)
    sql += ` LIMIT ${limit} OFFSET ${offset}`;

    // 4. Execution -> Arrow IPC
    // queryToArrowIPC returns a raw Uint8Array. 
    // This is much faster than JSON serialization for passing to main thread.
    const result = await conn.queryToArrowIPC(sql);

    // Transfer the buffer to main thread (Zero-Copy transfer)
    postMessage({ 
        type: 'RESULT', 
        data: result.buffer 
    }, [result.buffer]); // Transferable
};

// Message Router
onmessage = async (e) => {
    const { type, payload } = e.data;

    try {
        switch (type) {
            case 'INIT':
                await init();
                break;
            case 'LOAD_DATA':
                await loadJSON(payload);
                break;
            case 'QUERY':
                await runQuery(payload);
                break;
        }
    } catch (err) {
        console.error("DuckDB Worker Error:", err);
        postMessage({ type: 'ERROR', message: err.message });
    }
};
