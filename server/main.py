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
                data = orjson.loads(f.read())
            
            album_source = data.get("album", {})
            tracks_source = data.get("tracks", [])
            
            alb_id = album_source.get("album_root_path")
            if not alb_id:
                return None

            excluded = {"metadata_toml_hash", "metadata_toml_mtime", "lock_hash"}
            clean_album = {k: v for k, v in album_source.items() if k not in excluded}
            clean_album["id"] = alb_id
            clean_album["tracks"] = tracks_source
            
            track_mapping = {}
            for t in tracks_source:
                t_id = t.get("track_library_path")
                t_path_rel = t.get("track_path")
                if t_id and t_path_rel:
                    track_mapping[t_id] = t_path_rel 

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
            
            self.albums.append(album_data)
            alb_id = album_data["id"]
            self.album_map[alb_id] = album_data
            
            for t_id, t_rel in t_map.items():
                full_path = LIBRARY_ROOT / alb_id / t_rel
                self.track_map[t_id] = str(full_path)
            
            count += 1

        self.albums.sort(key=lambda x: x["id"])
        print(f"Live Lake Initialized: {count} albums in RAM.")

    def update_album(self, folder_path_str: str):
        """
        Hot-reloads a single album from disk.
        """
        folder_path = Path(folder_path_str)
        lock_path = folder_path / "metadata.lock.json"
        
        if not lock_path.exists():
            print(f"Update failed: {lock_path} not found")
            return None
            
        res = self._parse_lock_file(lock_path)
        if not res: 
            return None
            
        new_album, new_t_map = res
        alb_id = new_album["id"]
        
        existing = self.album_map.get(alb_id)
        
        if existing:
            existing.clear()
            existing.update(new_album)
        else:
            self.albums.append(new_album)
            self.album_map[alb_id] = new_album
            
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
                pass

manager = ConnectionManager()

# --- Endpoints ---

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await manager.connect(websocket)
    try:
        payload = {
            "type": "INIT",
            "data": STATE.albums
        }
        await websocket.send_bytes(orjson.dumps(payload))
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        manager.disconnect(websocket)

@app.post("/api/internal/reload")
async def trigger_reload(path: str):
    if not path:
        raise HTTPException(status_code=400)
    print(f"Hot Reload Request: {path}")
    updated_album = STATE.update_album(path)
    if updated_album:
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
        raise HTTPException(status_code=500)
    file_path = (THUMBNAIL_ROOT / f"{cover_hash}.png").resolve()
    if not str(file_path).startswith(str(THUMBNAIL_ROOT)) or not file_path.exists():
         raise HTTPException(status_code=404)
    return FileResponse(file_path, headers={"Cache-Control": "public, max-age=31536000"})

@app.get("/api/assets/{album_id:path}/cover")
def get_album_cover(album_id: str):
    album = STATE.album_map.get(album_id)
    if not album or not album.get("cover_path") or album.get("cover_path") == "default_cover.png":
        raise HTTPException(status_code=404)
    file_path = (LIBRARY_ROOT / album_id / album["cover_path"]).resolve()
    if not str(file_path).startswith(str(LIBRARY_ROOT)) or not file_path.exists():
         raise HTTPException(status_code=404)
    return FileResponse(file_path)

@app.post("/api/play/{album_id:path}")
def play_album(album_id: str):
    """
    Retrieves the ordered tracklist from the Live Lake, clears the current
    MPD queue, and starts playback of the selected album.
    """
    album = STATE.album_map.get(album_id)
    if not album:
        raise HTTPException(status_code=404, detail="Album not found")
    
    paths_to_queue = []
    # Iterating through album['tracks'] preserves the Natural Order 
    # established by the Compiler (Phase 3/4).
    for t in album.get("tracks", []):
        t_id = t.get("track_library_path")
        if t_id in STATE.track_map:
            paths_to_queue.append(STATE.track_map[t_id])
            
    if not paths_to_queue:
        raise HTTPException(status_code=404, detail="No tracks found for this album")
        
    _send_to_mpd(paths_to_queue)
    return {"status": "ok", "count": len(paths_to_queue)}

def _send_to_mpd(paths):
    """
    Internal helper to communicate with MPD. 
    Clears current queue and appends new tracks.
    """
    client = MPDClient()
    try:
        client.connect("localhost", 6600)
        client.clear()
        for p in paths:
            # Resolve absolute server path to a path relative to MPD's music root
            rel_p = str(Path(p).relative_to(LIBRARY_ROOT))
            client.add(rel_p)
        client.play()
        client.close()
    except Exception as e:
        print(f"MPD Error: {e}")
        raise HTTPException(status_code=500, detail=f"MPD Connection Failed: {e}")

if __name__ == "__main__":
    uvicorn.run("server.main:app", host="127.0.0.1", port=8000, log_level="info")
