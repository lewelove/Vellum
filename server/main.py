import sys
import uvicorn
import json
import tomllib
import orjson
import asyncio
import concurrent.futures
from contextlib import asynccontextmanager
from pathlib import Path
from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect
from fastapi.responses import FileResponse
from fastapi.middleware.cors import CORSMiddleware
from mpd import MPDClient

# --- Configuration & Globals ---

CONFIG = {}
LIBRARY_ROOT = None
THUMBNAIL_ROOT = None

def load_config():
    config_path = Path("config.toml")
    if not config_path.exists():
        return {}
    with open(config_path, "rb") as f:
        return tomllib.load(f)

# --- The Live Lake (State Manager) ---

class LibraryState:
    def __init__(self):
        self.albums = []  # List[dict]
        self.album_map = {}  # id -> dict (Reference to object in self.albums)
        self.track_map = {}  # track_library_path -> absolute_path_str

    def _parse_lock_file(self, lock_path: Path):
        """
        Worker function for parallel execution.
        Reads and prepares a single album object from a lock file.
        """
        try:
            with open(lock_path, "rb") as f:
                # orjson is faster than json.load
                data = orjson.loads(f.read())
            
            album_source = data.get("album", {})
            tracks_source = data.get("tracks", [])
            
            # 1. Resolve ID
            # Use the ID from the file, or fallback to folder path
            alb_id = album_source.get("album_root_path")
            if not alb_id:
                # Fallback: relative path from library root
                # Note: We can't easily calc relative path here without passing LIBRARY_ROOT
                # so we rely on the lock file having 'album_root_path' correctly set by compiler.
                # If missing, this might be fragile, but the compiler ensures it.
                return None

            # 2. Cleanup
            # Exclude hash keys to save RAM/Network
            excluded = {"metadata_toml_hash", "metadata_toml_mtime", "lock_hash"}
            clean_album = {k: v for k, v in album_source.items() if k not in excluded}
            clean_album["id"] = alb_id
            clean_album["tracks"] = tracks_source
            
            # 3. Track Mapping Data (for MPD)
            track_mapping = {}
            for t in tracks_source:
                t_id = t.get("track_library_path")
                t_path_rel = t.get("track_path")
                if t_id and t_path_rel:
                    track_mapping[t_id] = t_path_rel # Relative to album

            return (clean_album, track_mapping)
            
        except Exception as e:
            print(f"Error loading {lock_path}: {e}")
            return None

    async def initialize(self):
        """
        Scans disk in parallel to build the initial state.
        """
        print(f"Scanning library at {LIBRARY_ROOT}...")
        
        loop = asyncio.get_running_loop()
        lock_files = list(LIBRARY_ROOT.rglob("metadata.lock.json"))
        
        results = []
        # Use a ProcessPool or ThreadPool. ThreadPool is usually fine for IO bound, 
        # but parsing huge JSONs might benefit from ProcessPool. 
        # However, ThreadPool is safer with sharing memory. orjson releases GIL often.
        with concurrent.futures.ThreadPoolExecutor() as pool:
            results = await loop.run_in_executor(
                None, 
                lambda: list(pool.map(self._parse_lock_file, lock_files))
            )
            
        self.albums = []
        self.album_map = {}
        self.track_map = {}
        
        count = 0
        for res in results:
            if not res: continue
            album_data, t_map = res
            
            # Insert into main list
            self.albums.append(album_data)
            
            # Indexing
            alb_id = album_data["id"]
            self.album_map[alb_id] = album_data
            
            # Track Indexing
            for t_id, t_rel in t_map.items():
                # We store full path string for MPD
                full_path = LIBRARY_ROOT / alb_id / t_rel
                self.track_map[t_id] = str(full_path)
            
            count += 1

        # Sort by ID to ensure consistent initial order
        self.albums.sort(key=lambda x: x["id"])
        print(f"Live Lake Initialized: {count} albums in RAM.")

    def update_album(self, folder_path_str: str):
        """
        Hot-reloads a single album from disk.
        Returns the new album object or None.
        """
        folder_path = Path(folder_path_str)
        lock_path = folder_path / "metadata.lock.json"
        
        if not lock_path.exists():
            # Handle Deletion? 
            # For now, we assume this is an update/creation trigger.
            print(f"Update failed: {lock_path} not found")
            return None
            
        res = self._parse_lock_file(lock_path)
        if not res: 
            return None
            
        new_album, new_t_map = res
        alb_id = new_album["id"]
        
        # 1. Update List
        # If it exists, replace it. If not, append.
        existing = self.album_map.get(alb_id)
        
        if existing:
            # In-place update of the dictionary object
            # This preserves the reference in self.albums list?
            # No, self.albums contains references. We must update the content of the dict
            # or replace the reference in the list.
            # Replacing reference in list is O(N). Updating dict content is O(1).
            existing.clear()
            existing.update(new_album)
        else:
            self.albums.append(new_album)
            self.album_map[alb_id] = new_album
            # Resorting the whole list might be expensive. 
            # We append for now. Client sorts anyway.
            
        # 2. Update Tracks
        # Pruning old tracks for this album is hard without a reverse index.
        # But we can just overwrite keys in track_map.
        for t_id, t_rel in new_t_map.items():
            full_path = LIBRARY_ROOT / alb_id / t_rel
            self.track_map[t_id] = str(full_path)
            
        return new_album

STATE = LibraryState()

# --- Lifecycle ---

@asynccontextmanager
async def lifespan(app: FastAPI):
    global CONFIG, LIBRARY_ROOT, THUMBNAIL_ROOT, STATE
    
    CONFIG = load_config()
    root_str = CONFIG.get("storage", {}).get("library_root")
    thumb_str = CONFIG.get("storage", {}).get("thumbnail_cache_folder")
    
    if root_str:
        LIBRARY_ROOT = Path(root_str).expanduser().resolve()
    if thumb_str:
        THUMBNAIL_ROOT = Path(thumb_str).expanduser().resolve()
        
    if LIBRARY_ROOT:
        await STATE.initialize()
        
    yield

app = FastAPI(lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# --- WebSocket Manager ---

class ConnectionManager:
    def __init__(self):
        self.active_connections: list[WebSocket] = []

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.active_connections.append(websocket)

    def disconnect(self, websocket: WebSocket):
        if websocket in self.active_connections:
            self.active_connections.remove(websocket)

    async def broadcast_bytes(self, data: bytes):
        for connection in self.active_connections:
            try:
                await connection.send_bytes(data)
            except Exception:
                # If send fails, connection likely dead
                pass

manager = ConnectionManager()

# --- Endpoints ---

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await manager.connect(websocket)
    try:
        # 1. Send Initial State (INIT)
        # We use orjson directly for speed
        payload = {
            "type": "INIT",
            "data": STATE.albums
        }
        await websocket.send_bytes(orjson.dumps(payload))
        
        # 2. Keep Alive
        while True:
            # Wait for messages (ignore them, just keep connection open)
            await websocket.receive_text()
            
    except WebSocketDisconnect:
        manager.disconnect(websocket)

@app.post("/api/internal/reload")
async def trigger_reload(path: str):
    """
    Called by CLI to notify of file system changes.
    """
    if not path:
        raise HTTPException(status_code=400)
        
    print(f"Hot Reload Request: {path}")
    
    updated_album = STATE.update_album(path)
    
    if updated_album:
        # Broadcast Delta (UPDATE)
        message = {
            "type": "UPDATE",
            "id": updated_album["id"],
            "payload": updated_album
        }
        await manager.broadcast_bytes(orjson.dumps(message))
        return {"status": "reloaded", "id": updated_album["id"]}
    else:
        raise HTTPException(status_code=404, detail="Could not reload album")

@app.get("/api/covers/{cover_hash}.png")
def get_cover_thumbnail(cover_hash: str):
    if not THUMBNAIL_ROOT:
        raise HTTPException(status_code=500, detail="Thumbnail config missing")
        
    file_path = (THUMBNAIL_ROOT / f"{cover_hash}.png").resolve()
    
    if not str(file_path).startswith(str(THUMBNAIL_ROOT)):
         raise HTTPException(status_code=403)

    if not file_path.exists():
        raise HTTPException(status_code=404)
        
    return FileResponse(
        file_path,
        headers={"Cache-Control": "public, max-age=31536000"}
    )

@app.get("/api/assets/{album_id:path}/cover")
def get_album_cover(album_id: str):
    if not LIBRARY_ROOT:
        raise HTTPException(status_code=500)
    
    album = STATE.album_map.get(album_id)
    if not album:
        raise HTTPException(status_code=404)
        
    cover_rel = album.get("cover_path")
    if not cover_rel or cover_rel == "default_cover.png":
        raise HTTPException(status_code=404)
        
    file_path = (LIBRARY_ROOT / album_id / cover_rel).resolve()
    if not str(file_path).startswith(str(LIBRARY_ROOT)):
         raise HTTPException(status_code=403)

    if not file_path.exists():
        raise HTTPException(status_code=404)
        
    return FileResponse(file_path)

@app.post("/api/play/{album_id:path}")
def play_album(album_id: str):
    # Retrieve tracks from STATE.track_map using the album_id prefix
    # or iterate tracks in the album object
    
    album = STATE.album_map.get(album_id)
    if not album:
        raise HTTPException(status_code=404, detail="Album not found")
    
    paths_to_queue = []
    
    # Efficient lookup: The album object already contains the track list
    # We just need to resolve them to absolute paths
    for t in album.get("tracks", []):
        t_id = t.get("track_library_path")
        if t_id in STATE.track_map:
            paths_to_queue.append(STATE.track_map[t_id])
            
    if not paths_to_queue:
        raise HTTPException(status_code=404, detail="No tracks found")
        
    _send_to_mpd(paths_to_queue)
    return {"status": "ok", "count": len(paths_to_queue)}

def _send_to_mpd(paths):
    client = MPDClient()
    try:
        client.connect("localhost", 6600)
        client.clear()
        for p in paths:
            # MPD expects paths relative to its own music_directory
            # Assuming MPD music_directory == LIBRARY_ROOT
            rel_p = str(Path(p).relative_to(LIBRARY_ROOT))
            client.add(rel_p)
        client.play()
        client.close()
    except Exception as e:
        print(f"MPD Error: {e}")
        raise HTTPException(status_code=500, detail="MPD Connection Failed")

if __name__ == "__main__":
    uvicorn.run("server.main:app", host="127.0.0.1", port=8000, log_level="info")
