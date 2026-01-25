import sys
import uvicorn
import json
import tomllib
import orjson
import asyncio
import concurrent.futures
from contextlib import asynccontextmanager
from pathlib import Path, PurePosixPath
from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect
from fastapi.responses import FileResponse
from fastapi.middleware.cors import CORSMiddleware
from mpd import MPDClient, ConnectionError

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
        self.albums = []  
        self.album_map = {}  
        self.track_map = {}  
        self.path_lookup = {} 

    def _normalize(self, path_str: str) -> str:
        if not path_str: return ""
        return path_str.lstrip('/')

    def _parse_lock_file(self, lock_path: Path):
        try:
            with open(lock_path, "rb") as f:
                data = orjson.loads(f.read())
            
            album_source = data.get("album", {})
            tracks_source = data.get("tracks", [])
            alb_id = album_source.get("album_root_path")
            
            if not alb_id: return None

            excluded = {"metadata_toml_hash", "metadata_toml_mtime", "lock_hash"}
            clean_album = {k: v for k, v in album_source.items() if k not in excluded}
            clean_album["id"] = alb_id
            clean_album["tracks"] = tracks_source
            
            t_map, p_map = {}, {}
            for t in tracks_source:
                t_id = t.get("track_library_path")
                t_path_rel = t.get("track_path")
                if t_id and t_path_rel:
                    t_map[t_id] = t_path_rel 
                    full_rel = self._normalize(str(PurePosixPath(alb_id) / t_path_rel))
                    p_map[full_rel] = alb_id

            return (clean_album, t_map, p_map)
        except Exception:
            return None

    async def initialize(self):
        print(f"Scanning library at {LIBRARY_ROOT}...")
        loop = asyncio.get_running_loop()
        lock_files = list(LIBRARY_ROOT.rglob("metadata.lock.json"))
        
        with concurrent.futures.ThreadPoolExecutor() as pool:
            results = await loop.run_in_executor(None, lambda: list(pool.map(self._parse_lock_file, lock_files)))
            
        self.albums, self.album_map, self.track_map, self.path_lookup = [], {}, {}, {}
        for res in results:
            if not res: continue
            album_data, t_map, p_map = res
            self.albums.append(album_data)
            self.album_map[album_data["id"]] = album_data
            for t_id, t_rel in t_map.items():
                self.track_map[t_id] = str(LIBRARY_ROOT / album_data["id"] / t_rel)
            self.path_lookup.update(p_map)

        self.albums.sort(key=lambda x: x["id"])
        print(f"Live Lake Initialized: {len(self.albums)} albums. {len(self.path_lookup)} tracks.")

    def update_album(self, folder_path_str: str):
        res = self._parse_lock_file(Path(folder_path_str) / "metadata.lock.json")
        if not res: return None
        new_album, new_t_map, new_p_map = res
        alb_id = new_album["id"]
        existing = self.album_map.get(alb_id)
        if existing:
            existing.clear()
            existing.update(new_album)
        else:
            self.albums.append(new_album)
            self.album_map[alb_id] = new_album
        for t_id, t_rel in new_t_map.items():
            self.track_map[t_id] = str(LIBRARY_ROOT / alb_id / t_rel)
        self.path_lookup.update(new_p_map)
        return new_album

STATE = LibraryState()

# --- MPD Monitor ---

async def mpd_monitor_loop():
    """
    Non-blocking polling monitor.
    Uses 1s sleep instead of blocking idle() to prevent shutdown hangs on Linux.
    """
    client = MPDClient()
    print("Starting MPD Monitor...")
    
    while True:
        try:
            try:
                client.ping()
            except (ConnectionError, OSError):
                client.connect("localhost", 6600)

            status = client.status()
            current = client.currentsong()
            file_path = current.get("file")
            
            payload = {
                "type": "MPD_STATUS",
                "state": status.get("state"),
                "file": file_path,
                "album_id": STATE.path_lookup.get(STATE._normalize(file_path)),
                "elapsed": status.get("elapsed"),
                "duration": status.get("duration"),
                "title": current.get("title"),
                "artist": current.get("artist"),
            }
            await manager.broadcast_bytes(orjson.dumps(payload))
            await asyncio.sleep(1) # Non-blocking poll
            
        except (ConnectionError, OSError):
            await asyncio.sleep(2)
        except asyncio.CancelledError:
            try: client.disconnect()
            except: pass
            break
        except Exception:
            await asyncio.sleep(1)

# --- Lifecycle ---

@asynccontextmanager
async def lifespan(app: FastAPI):
    global CONFIG, LIBRARY_ROOT, THUMBNAIL_ROOT
    CONFIG = load_config()
    root_str = CONFIG.get("storage", {}).get("library_root")
    thumb_str = CONFIG.get("storage", {}).get("thumbnail_cache_folder")
    if root_str: LIBRARY_ROOT = Path(root_str).expanduser().resolve()
    if thumb_str: THUMBNAIL_ROOT = Path(thumb_str).expanduser().resolve()
    if LIBRARY_ROOT: await STATE.initialize()
    
    monitor_task = asyncio.create_task(mpd_monitor_loop())
    yield
    monitor_task.cancel()
    try: await asyncio.wait_for(monitor_task, timeout=2)
    except Exception: pass

app = FastAPI(lifespan=lifespan)
app.add_middleware(CORSMiddleware, allow_origins=["*"], allow_credentials=True, allow_methods=["*"], allow_headers=["*"])

class ConnectionManager:
    def __init__(self): self.active_connections = []
    async def connect(self, ws): await ws.accept(); self.active_connections.append(ws)
    def disconnect(self, ws): 
        if ws in self.active_connections: self.active_connections.remove(ws)
    async def broadcast_bytes(self, data):
        for c in self.active_connections:
            try: await c.send_bytes(data)
            except: pass

manager = ConnectionManager()

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await manager.connect(websocket)
    try:
        await websocket.send_bytes(orjson.dumps({"type": "INIT", "data": STATE.albums}))
        while True: await websocket.receive_text()
    except WebSocketDisconnect: manager.disconnect(websocket)

@app.post("/api/internal/reload")
async def trigger_reload(path: str):
    updated = STATE.update_album(path)
    if updated:
        await manager.broadcast_bytes(orjson.dumps({"type": "UPDATE", "id": updated["id"], "payload": updated}))
        return {"status": "reloaded"}
    raise HTTPException(status_code=404)

@app.get("/api/covers/{cover_hash}.png")
def get_cover_thumbnail(cover_hash: str):
    path = (THUMBNAIL_ROOT / f"{cover_hash}.png").resolve()
    if not path.exists(): raise HTTPException(status_code=404)
    return FileResponse(path, headers={"Cache-Control": "public, max-age=31536000"})

@app.get("/api/assets/{album_id:path}/cover")
def get_album_cover(album_id: str):
    album = STATE.album_map.get(album_id)
    if not album or not album.get("cover_path") or album.get("cover_path") == "default_cover.png":
        raise HTTPException(status_code=404)
    path = (LIBRARY_ROOT / album_id / album["cover_path"]).resolve()
    if not path.exists(): raise HTTPException(status_code=404)
    return FileResponse(path)

@app.post("/api/play/{album_id:path}")
def play_album(album_id: str):
    album = STATE.album_map.get(album_id)
    if not album: raise HTTPException(status_code=404)
    paths = [STATE.track_map[t["track_library_path"]] for t in album.get("tracks", []) if t.get("track_library_path") in STATE.track_map]
    if not paths: raise HTTPException(status_code=404)
    client = MPDClient()
    try:
        client.connect("localhost", 6600); client.clear()
        for p in paths: client.add(str(Path(p).relative_to(LIBRARY_ROOT)))
        client.play(); client.close()
    except Exception as e: raise HTTPException(status_code=500, detail=str(e))
    return {"status": "ok"}

if __name__ == "__main__":
    uvicorn.run("server.main:app", host="127.0.0.1", port=8000, log_level="info")
