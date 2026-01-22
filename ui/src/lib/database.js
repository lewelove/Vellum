import { tableFromIPC } from 'apache-arrow';
import Worker from '../workers/db.worker.js?worker'; // Vite syntax for worker import

class DatabaseClient {
    constructor() {
        this.worker = null;
        this.readyPromise = null;
        this.listeners = new Set();
        this.pendingRequests = new Map();
    }

    // Initialize the Worker
    async init() {
        if (this.worker) return;

        this.worker = new Worker();
        
        // Handle responses from Worker
        this.worker.onmessage = (event) => {
            const { type, data, message } = event.data;

            if (type === 'RESULT') {
                // 1. Inflate Arrow Buffer to JS Objects
                // This is extremely fast even for large datasets
                const arrowTable = tableFromIPC(data);
                const rows = arrowTable.toArray().map(row => row.toJSON());
                
                // Notify subscribers (your UI)
                this.listeners.forEach(cb => cb(rows));
            } 
            else if (type === 'UPDATED') {
                console.log('Database updated successfully');
            }
            else if (type === 'ERROR') {
                console.error('DB Error:', message);
            }
        };

        this.worker.postMessage({ type: 'INIT' });
    }

    // Load or "Patch" data
    // Accepts the raw merged JSON string
    async setData(jsonString) {
        if (!this.worker) await this.init();
        this.worker.postMessage({ type: 'LOAD_DATA', payload: jsonString });
    }

    // Main Query Interface
    // filter: { genre: 'rock' }
    // sort: { column: 'year', direction: 'DESC' }
    async search(filter, sort, limit = 100, offset = 0) {
        if (!this.worker) await this.init();
        
        this.worker.postMessage({ 
            type: 'QUERY', 
            payload: { filter, sort, limit, offset } 
        });
    }

    // React/Vue hook subscription helper
    subscribe(callback) {
        this.listeners.add(callback);
        return () => this.listeners.delete(callback);
    }
}

// Export Singleton
export const db = new DatabaseClient();
