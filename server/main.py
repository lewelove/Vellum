import uvicorn
import sqlite3
import hashlib
import tomllib
import urllib.parse
from pathlib import Path
from fastapi import FastAPI, HTTPException
from fastapi.responses import FileResponse
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# --- Configuration & Database ---

def load_config():
    config_path = Path("config.toml")
    if not config_path.exists():
        return {}
    with open(config_path, "rb") as f:
        return tomllib.load(f)

CONFIG = load_config()

def get_db_path():
    # Default to ~/.mpf2k/library.db
    return Path("~/.mpf2k/library.db").expanduser().resolve()

def get_library_root():
    root = CONFIG.get("storage", {}).get("library_root")
    if root:
        return Path(root).expanduser().resolve()
    return None

def get_db_connection():
    db_path = get_db_path()
    if not db_path.exists():
        raise HTTPException(status_code=500, detail="Database not found. Run 'mpf2k update' first.")
    
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    conn.row_factory = sqlite3.Row
    return conn

def generate_color(album_id: str) -> str:
    # Deterministic color generation based on album ID (path)
    palette = ["#EA4335", "#34A853", "#FBBC04", "#4285F4", "#A142F4", "#F4426C", "#42F4E2"]
    # Use MD5 for stability across runs/platforms
    hash_val = int(hashlib.md5(album_id.encode("utf-8")).hexdigest(), 16)
    return palette[hash_val % len(palette)]

# --- Endpoints ---

@app.get("/api/library")
def get_library():
    conn = get_db_connection()
    try:
        # Fetch basic shelf info
        # We need id (path), title (ALBUM), artist (ALBUMARTIST), totaltracks (TOTALTRACKS)
        # Sort by date_added DESC if available, else standard
        cursor = conn.execute("SELECT id, ALBUM, ALBUMARTIST, TOTALTRACKS, date_added FROM albums ORDER BY date_added DESC, ALBUM ASC")
        rows = cursor.fetchall()
        
        albums = []
        for row in rows:
            albums.append({
                "id": row["id"],
                "title": row["ALBUM"],
                "artist": row["ALBUMARTIST"],
                "totalTracks": int(row["TOTALTRACKS"] or 0),
                "color": generate_color(row["id"]),
                "tracks": None # Lazy loaded later
            })
        return albums
    finally:
        conn.close()

@app.get("/api/album/{album_id:path}")
def get_album_tracks(album_id: str):
    conn = get_db_connection()
    try:
        # Fetch inflated tracks
        # Order by Disc, Track
        cursor = conn.execute("""
            SELECT * FROM tracks 
            WHERE album_id = ? 
            ORDER BY 
                CAST(DISCNUMBER AS INTEGER), 
                CAST(TRACKNUMBER AS INTEGER)
        """, (album_id,))
        
        rows = cursor.fetchall()
        tracks = []
        for row in rows:
            # Convert row to dict
            track = dict(row)
            # Normalize title for UI if needed (though UI uses string interpolation of object)
            tracks.append(track)
            
        return tracks
    finally:
        conn.close()

@app.get("/api/assets/{album_id:path}/cover")
def get_album_cover(album_id: str):
    lib_root = get_library_root()
    if not lib_root:
        raise HTTPException(status_code=500, detail="Library root not configured.")

    conn = get_db_connection()
    try:
        cursor = conn.execute("SELECT cover_path FROM albums WHERE id = ?", (album_id,))
        row = cursor.fetchone()
        
        if not row:
            raise HTTPException(status_code=404, detail="Album not found")
            
        cover_rel = row["cover_path"]
        if not cover_rel or cover_rel == "default_cover.png":
            # Fallback or handle default
            # For now, 404 and let UI show color
            raise HTTPException(status_code=404, detail="No cover art")
            
        # Construct absolute path
        # album_id is relative to lib_root. cover_rel is relative to album_id (album root).
        # WAIT: In resolver logic:
        #   cover_path is relative to metadata.toml (which is album root)
        #   album_id IS the album root relative to library_root
        
        abs_path = lib_root / album_id / cover_rel
        
        if not abs_path.exists():
            raise HTTPException(status_code=404, detail="Cover file missing")
            
        return FileResponse(abs_path)
        
    finally:
        conn.close()

@app.post("/api/play/{album_id:path}")
def play_album(album_id: str):
    conn = get_db_connection()
    try:
        # Get all tracks for album
        cursor = conn.execute("""
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
            abs_path = lib_root / rel_path
            files_to_play.append(str(abs_path))
            print(f"ADD: {abs_path}")
            
        # Here we would use python-mpd2 to clear and add
        return {"status": "success", "message": f"Queued {len(files_to_play)} tracks"}
        
    finally:
        conn.close()

if __name__ == "__main__":
    uvicorn.run("server.main:app", host="127.0.0.1", port=8000, reload=True)
