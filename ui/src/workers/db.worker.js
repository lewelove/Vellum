import * as duckdb from '@duckdb/duckdb-wasm';
import { ConsoleLogger } from '@duckdb/duckdb-wasm';

// THIS IS DEAD CODE CURRENTLY
// NEVER USED ANYWHERE IN CODEBASE

const JSDELIVR_BUNDLES = duckdb.getJsDelivrBundles();

let db = null;
let conn = null;

const init = async () => {
    if (db) return;

    const bundle = await duckdb.selectBundle(JSDELIVR_BUNDLES);
    
    const worker = await duckdb.createWorker(bundle.mainWorker);
    const logger = new ConsoleLogger();

    db = new duckdb.AsyncDuckDB(logger, worker);
    await db.instantiate(bundle.mainModule, bundle.pthreadWorker);

    conn = await db.connect();
    
    postMessage({ type: 'READY' });
};

const loadJSON = async (jsonString) => {
    if (!db || !conn) await init();

    await db.registerFileText('import.json', jsonString);

    await conn.query(`
        CREATE OR REPLACE TABLE library AS 
        SELECT * FROM read_json_auto('import.json')
    `);

    await db.registerFileText('import.json', '');
    
    postMessage({ type: 'UPDATED' });
};

const runQuery = async ({ filter, sort, limit = 50, offset = 0 }) => {
    if (!conn) return;

    let sql = `SELECT * FROM library`;
    
    const clauses = [];
    if (filter) {
        Object.entries(filter).forEach(([key, value]) => {
            if (typeof value === 'string') {
                clauses.push(`${key} ILIKE '%${value}%'`);
            } else {
                clauses.push(`${key} = ${value}`);
            }
        });
    }

    if (clauses.length > 0) {
        sql += ` WHERE ${clauses.join(' AND ')}`;
    }

    if (sort && sort.column) {
        sql += ` ORDER BY ${sort.column} ${sort.direction || 'ASC'}`;
    }

    sql += ` LIMIT ${limit} OFFSET ${offset}`;

    const result = await conn.queryToArrowIPC(sql);

    postMessage({ 
        type: 'RESULT', 
        data: result.buffer 
    }, [result.buffer]); // Transferable
};

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
