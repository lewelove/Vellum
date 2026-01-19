import uvicorn
import sqlite3
import hashlib
import json
import tomllib
from contextlib import asynccontextmanager
from pathlib import Path
from fastapi import FastAPI, HTTPException, Query
from fastapi.responses import FileResponse
from fastapi.middleware.cors import CORSMiddleware
from typing import Optional

# --- Global State ---
CONFIG = {}
MEM_CONN = None
VALID_ALBUM_COLUMNS = set()

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
    4. Introspects the schema for safe dynamic querying.
    """
    global MEM_CONN, VALID_ALBUM_COLUMNS
    
    db_path = Path("~/.mpf2k/library.db").expanduser().resolve()
    
    # Connect to memory
    MEM_CONN = sqlite3.connect(":memory:", check_same_thread=False)
    MEM_CONN.row_factory = sqlite3.Row

    if db_path.exists():
        print(f"Loading database from {db_path} into RAM...")
        try:
            disk_conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
            disk_conn.backup(MEM_CONN)
            disk_conn.close()
            print("Database loaded into memory.")
        except Exception as e:
            print(f"Error loading database: {e}")
    else:
        print(f"Warning: Database at {db_path} not found. Starting empty.")

    # Introspect Schema
    try:
        cursor = MEM_CONN.execute("PRAGMA table_info(albums)")
        VALID_ALBUM_COLUMNS = {row["name"] for row in cursor.fetchall()}
        print(f"Schema Introspection: Found {len(VALID_ALBUM_COLUMNS)} album columns.")
    except Exception:
        VALID_ALBUM_COLUMNS = set()

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
def get_library(
    group_by: Optional[str] = None,
    filter_col: Optional[str] = None,
    filter_val: Optional[str] = None,
    sort_col: str = "date_added",
    sort_dir: str = "DESC"
):
    """
    The Query Factory.
    Handles Grouping, Filtering, and Grid Generation.
    Returns:
      - List[dict] for Grouping (if group_by is set)
      - List[AlbumCustodian] for Grid (default)
    """
    if not MEM_CONN:
        return []

    # --- MODE 1: GROUPING (Sidebar) ---
    if group_by:
        if group_by not in VALID_ALBUM_COLUMNS:
            raise HTTPException(status_code=400, detail=f"Invalid group column: {group_by}")
        
        try:
            # Fast distinct aggregation
            sql = f"""
                SELECT "{group_by}" as value, COUNT(*) as count 
                FROM albums 
                WHERE "{group_by}" != '' AND "{group_by}" IS NOT NULL
                GROUP BY "{group_by}" 
                ORDER BY "{group_by}" ASC
            """
            cursor = MEM_CONN.execute(sql)
            return [dict(row) for row in cursor.fetchall()]
        except Exception as e:
            print(f"Grouping Error: {e}")
            raise HTTPException(status_code=500, detail=str(e))

    # --- MODE 2: GRID VIEW (Albums + Tracks) ---
    
    # 1. Input Sanitization
    final_sort_col = sort_col if sort_col in VALID_ALBUM_COLUMNS else "date_added"
    final_sort_dir = "DESC" if sort_dir.upper() == "DESC" else "ASC"
    
    where_clause = ""
    params = []
    
    if filter_col and filter_val:
        if filter_col in VALID_ALBUM_COLUMNS:
            where_clause = f'WHERE "{filter_col}" = ?'
            params.append(filter_val)
    
    # 2. SQL Construction
    # We fetch the album row AND a subquery JSON blob for tracks
    sql = f"""
        SELECT 
            a.id,
            a.ALBUM,
            a.ALBUMARTIST,
            a.cover_path,
            (
                SELECT json_group_array(json_object(
                    'number', t.TRACKNUMBER,
                    'title', t.TITLE,
                    'duration', t.track_duration_time,
                    'path', t.track_library_path
                ))
                FROM (
                    SELECT * FROM tracks tr 
                    WHERE tr.album_id = a.id
                    ORDER BY CAST(tr.DISCNUMBER AS INTEGER), CAST(tr.TRACKNUMBER AS INTEGER)
                ) t
            ) as tracks_json
        FROM albums a
        {where_clause}
        ORDER BY "{final_sort_col}" {final_sort_dir}, a.ALBUM ASC
    """

    try:
        cursor = MEM_CONN.execute(sql, params)
        rows = cursor.fetchall()
        
        albums = []
        for row in rows:
            # 3. DTO Transformation ("Pixel-Ready")
            try:
                tracks_list = json.loads(row["tracks_json"])
            except (ValueError, TypeError):
                tracks_list = []

            albums.append({
                "id": row["id"],
                "title": row["ALBUM"],          # Map ALBUM -> title
                "artist": row["ALBUMARTIST"],   # Map ALBUMARTIST -> artist
                "color": generate_color(row["id"]),
                "cover_path": row["cover_path"],
                "tracks": tracks_list
            })
            
        return albums

    except Exception as e:
        print(f"Grid Query Error: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/assets/{album_id:path}/cover")
def get_album_cover(album_id: str):
    lib_root = get_library_root()
    if not lib_root:
        raise HTTPException(status_code=500, detail="Library root not configured.")

    if not MEM_CONN:
        raise HTTPException(status_code=503, detail="DB not initialized")

    try:
        cursor = MEM_CONN.execute("SELECT cover_path FROM albums WHERE id = ?", (album_id,))
        row = cursor.fetchone()
        
        if not row:
            # Fallback for consistency
            cursor = MEM_CONN.execute("SELECT cover_path FROM tracks WHERE album_id = ? LIMIT 1", (album_id,))
            row = cursor.fetchone()
        
        if not row:
            raise HTTPException(status_code=404, detail="Album not found")
            
        cover_rel = row["cover_path"]
        if not cover_rel or cover_rel == "default_cover.png":
            raise HTTPException(status_code=404, detail="No cover art")
            
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
            
        return {"status": "success", "message": f"Queued {len(files_to_play)} tracks"}
        
    except Exception as e:
        print(f"Play Error: {e}")
        return {"status": "error", "message": str(e)}

if __name__ == "__main__":
    uvicorn.run("server.main:app", host="127.0.0.1", port=8000, reload=True)
