import uvicorn
import sqlite3
import hashlib
import json
import tomllib
from contextlib import asynccontextmanager
from pathlib import Path
from fastapi import FastAPI, HTTPException
from fastapi.responses import FileResponse
from fastapi.middleware.cors import CORSMiddleware

# --- Global State ---
CONFIG = {}
MEM_CONN = None

# --- Configuration & Database Helpers ---

def load_config():
    config_path = Path("config.toml")
    if not config_path.exists():
        return {}
    with open(config_path, "rb") as f:
        return tomllib.load(f)

def get_library_root():
    root = CONFIG.get("storage", {}).get("library_root")
    if root:
        return Path(root).expanduser().resolve()
    return None

def init_db():
    """
    Bootstraps the In-Memory Database.
    1. Connects to the disk database (ro).
    2. Creates an in-memory database.
    3. Copies the disk content to memory.
    """
    global MEM_CONN
    
    db_path = Path("~/.mpf2k/library.db").expanduser().resolve()
    if not db_path.exists():
        print(f"Warning: Database at {db_path} not found.")
        # Create an empty in-memory DB so the server doesn't crash, but it will be empty.
        MEM_CONN = sqlite3.connect(":memory:", check_same_thread=False)
        MEM_CONN.row_factory = sqlite3.Row
        return

    print(f"Loading database from {db_path} into RAM...")
    
    # Connect to disk
    disk_conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    
    # Connect to memory
    MEM_CONN = sqlite3.connect(":memory:", check_same_thread=False)
    MEM_CONN.row_factory = sqlite3.Row
    
    # Backup (Copy) Disk -> Memory
    disk_conn.backup(MEM_CONN)
    
    disk_conn.close()
    print("Database loaded into memory.")

def generate_color(album_id: str) -> str:
    palette = ["#EA4335", "#34A853", "#FBBC04", "#4285F4", "#A142F4", "#F4426C", "#42F4E2"]
    hash_val = int(hashlib.md5(album_id.encode("utf-8")).hexdigest(), 16)
    return palette[hash_val % len(palette)]

# --- Lifecycle ---

@asynccontextmanager
async def lifespan(app: FastAPI):
    global CONFIG
    CONFIG = load_config()
    init_db()
    yield
    if MEM_CONN:
        MEM_CONN.close()

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

@app.get("/api/library")
def get_library():
    """
    The Super Query.
    Aggregates tracks into albums using SQLite JSON logic.
    Returns fully inflated AlbumCustodian objects.
    """
    if not MEM_CONN:
        return []

    try:
        # We query the 'tracks' table because it is fully inflated (contains all album tags).
        # We group by album_id to fold tracks into a JSON array.
        
        # Subquery: Sorts tracks physically first so the array is ordered.
        sql = """
            SELECT 
                t.album_id as id,
                t.ALBUM as title,
                t.ALBUMARTIST as artist,
                t.cover_path,
                t.date_added,
                t.TOTALTRACKS as totalTracks,
                t.track_duration_time,
                json_group_array(
                    json_object(
                        'title', t.TITLE,
                        'path', t.track_library_path,
                        'number', t.TRACKNUMBER,
                        'disc', t.DISCNUMBER,
                        'duration', t.track_duration_time
                    )
                ) as tracks_json
            FROM (
                SELECT * FROM tracks 
                ORDER BY album_id, CAST(DISCNUMBER AS INTEGER), CAST(TRACKNUMBER AS INTEGER)
            ) t
            GROUP BY t.album_id
            ORDER BY t.date_added DESC, t.ALBUM ASC
        """
        
        cursor = MEM_CONN.execute(sql)
        rows = cursor.fetchall()
        
        albums = []
        for row in rows:
            # We must deserialise the JSON string from SQLite
            try:
                tracks_list = json.loads(row["tracks_json"])
            except:
                tracks_list = []

            albums.append({
                "id": row["id"],
                "title": row["title"],
                "artist": row["artist"],
                "cover_path": row["cover_path"],
                "totalTracks": row["totalTracks"], # Helper from DB
                "date_added": row["date_added"],
                "color": generate_color(row["id"]),
                "tracks": tracks_list
            })
            
        return albums
    except Exception as e:
        print(f"Query Error: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/assets/{album_id:path}/cover")
def get_album_cover(album_id: str):
    lib_root = get_library_root()
    if not lib_root:
        raise HTTPException(status_code=500, detail="Library root not configured.")

    if not MEM_CONN:
        raise HTTPException(status_code=503, detail="DB not initialized")

    try:
        # We can still query the memory DB for the path
        cursor = MEM_CONN.execute("SELECT cover_path FROM albums WHERE id = ?", (album_id,))
        row = cursor.fetchone()
        
        if not row:
            # Fallback: try finding it in tracks if albums table is missing/empty
            # (Though our arch guarantees consistency, safety first)
            cursor = MEM_CONN.execute("SELECT cover_path FROM tracks WHERE album_id = ? LIMIT 1", (album_id,))
            row = cursor.fetchone()
        
        if not row:
            raise HTTPException(status_code=404, detail="Album not found")
            
        cover_rel = row["cover_path"]
        if not cover_rel or cover_rel == "default_cover.png":
            raise HTTPException(status_code=404, detail="No cover art")
            
        # Construct absolute path
        # album_id is relative to lib_root
        abs_path = lib_root / album_id / cover_rel
        
        if not abs_path.exists():
            raise HTTPException(status_code=404, detail="Cover file missing")
            
        return FileResponse(abs_path)
        
    except Exception as e:
         raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/play/{album_id:path}")
def play_album(album_id: str):
    if not MEM_CONN:
        raise HTTPException(status_code=503, detail="DB not initialized")

    try:
        # Fetch tracks from memory DB
        cursor = MEM_CONN.execute("""
            SELECT track_library_path FROM tracks 
            WHERE album_id = ? 
            ORDER BY 
                CAST(DISCNUMBER AS INTEGER), 
                CAST(TRACKNUMBER AS INTEGER)
        """, (album_id,))
        
        rows = cursor.fetchall()
        if not rows:
             return {"status": "error", "message": "No tracks found"}
             
        lib_root = get_library_root()
        files_to_play = []
        
        print(f"--- Sending to MPD ({len(rows)} tracks) ---")
        for row in rows:
            rel_path = row["track_library_path"]
            if rel_path:
                abs_path = lib_root / rel_path
                files_to_play.append(str(abs_path))
                print(f"ADD: {abs_path}")
            
        # Placeholder for mpd2 integration
        return {"status": "success", "message": f"Queued {len(files_to_play)} tracks"}
        
    except Exception as e:
        print(f"Play Error: {e}")
        return {"status": "error", "message": str(e)}

if __name__ == "__main__":
    uvicorn.run("server.main:app", host="127.0.0.1", port=8000, reload=True)
