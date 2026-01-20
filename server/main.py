import sys
import uvicorn
import json
import tomllib
from contextlib import asynccontextmanager
from pathlib import Path
from fastapi import FastAPI, HTTPException, Body
from fastapi.responses import FileResponse
from fastapi.middleware.cors import CORSMiddleware
from mpd import MPDClient

# --- Global State ---
CONFIG = {}
LIBRARY_JSON_PATH = Path("~/.mpf2k/library.json").expanduser().resolve()
LIBRARY_ROOT = None
ALBUM_MAP = {} # album_id -> { cover_path, ... }
TRACK_MAP = {} # track_id -> absolute_path

# --- Helpers ---

def load_config():
    config_path = Path("config.toml")
    if not config_path.exists():
        return {}
    with open(config_path, "rb") as f:
        return tomllib.load(f)

def load_library_map():
    """
    Loads nested library.json into RAM to serve as a Lookup Table
    for Asset Serving and Playback resolution.
    """
    global ALBUM_MAP, TRACK_MAP
    
    if not LIBRARY_JSON_PATH.exists():
        print(f"Warning: {LIBRARY_JSON_PATH} not found. Run 'mpf2k build'.")
        return

    print("Loading Library Map...")
    with open(LIBRARY_JSON_PATH, "r", encoding="utf-8") as f:
        albums = json.load(f)
        
    for album in albums:
        a_id = album.get("id")
        
        # Build Album Map
        if a_id:
            ALBUM_MAP[a_id] = {
                "cover_path": album.get("cover_path")
            }
            
        # Build Track Map (Iterate into nested tracks)
        # Note: In the Nested Architecture, 'tracks' is a list inside the album object
        for t in album.get("tracks", []):
            t_id = t.get("track_library_path")
            t_path_rel = t.get("track_path")
            
            if t_id and t_path_rel:
                # Resolve absolute path for MPD
                # track_path is relative to the album folder (a_id)
                full_path = LIBRARY_ROOT / a_id / t_path_rel
                TRACK_MAP[t_id] = str(full_path)

# --- Lifecycle ---

@asynccontextmanager
async def lifespan(app: FastAPI):
    global CONFIG, LIBRARY_ROOT
    CONFIG = load_config()
    root_str = CONFIG.get("storage", {}).get("library_root")
    if root_str:
        LIBRARY_ROOT = Path(root_str).expanduser().resolve()
        
    load_library_map()
    yield

# --- App Definition ---

app = FastAPI(lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# --- Endpoints ---

@app.get("/library.json")
def get_library_json():
    """
    Serves the raw JSON database to the UI.
    """
    if not LIBRARY_JSON_PATH.exists():
        raise HTTPException(status_code=404, detail="Library artifact not found")
    return FileResponse(LIBRARY_JSON_PATH)

@app.get("/api/assets/{album_id:path}/cover")
def get_album_cover(album_id: str):
    if not LIBRARY_ROOT or album_id not in ALBUM_MAP:
        raise HTTPException(status_code=404)
        
    cover_rel = ALBUM_MAP[album_id].get("cover_path")
    if not cover_rel or cover_rel == "default_cover.png":
        raise HTTPException(status_code=404)
        
    # Security: Ensure path is within library
    file_path = (LIBRARY_ROOT / album_id / cover_rel).resolve()
    if not str(file_path).startswith(str(LIBRARY_ROOT)):
         raise HTTPException(status_code=403)

    if not file_path.exists():
        raise HTTPException(status_code=404)
        
    return FileResponse(file_path)

@app.post("/api/play/{album_id:path}")
def play_album(album_id: str):
    """
    Plays entire album by ID.
    Finds all tracks in TRACK_MAP belonging to this album.
    """
    paths_to_queue = []
    prefix = str(LIBRARY_ROOT / album_id)
    
    # Iterate all known tracks to find matches (Fast enough for single user)
    # Alternatively, we could have stored tracks inside ALBUM_MAP for faster lookup
    for t_id, abs_path in TRACK_MAP.items():
        if abs_path.startswith(prefix):
            paths_to_queue.append(abs_path)
            
    paths_to_queue.sort()

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
            rel_p = str(Path(p).relative_to(LIBRARY_ROOT))
            client.add(rel_p)
        client.play()
        client.close()
    except Exception as e:
        print(f"MPD Error: {e}")
        raise HTTPException(status_code=500, detail="MPD Connection Failed")

if __name__ == "__main__":
    uvicorn.run("server.main:app", host="127.0.0.1", port=8000)
