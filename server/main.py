import sys
print("DEBUG: Server script starting...", file=sys.stderr)

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

# New Modular Imports
from server.context import QueryContext
from server.registry import load_plugins, get_capabilities, get_grouper, get_filter, get_sorter

# --- Global State ---
CONFIG = {}
MEM_CONN = None
VALID_ALBUM_COLUMNS = set()

# --- Helpers ---

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
    global MEM_CONN, VALID_ALBUM_COLUMNS
    db_path = Path("~/.mpf2k/library.db").expanduser().resolve()
    
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

    try:
        cursor = MEM_CONN.execute("PRAGMA table_info(albums)")
        VALID_ALBUM_COLUMNS = {row["name"] for row in cursor.fetchall()}
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
    load_plugins() # Initialize the Registry
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

@app.get("/api/capabilities")
def get_api_capabilities():
    """Returns available filters, sorters, and groupers."""
    return get_capabilities()

@app.get("/api/sidebar/{group_key}")
def get_sidebar_group(group_key: str):
    """Executes a specific grouping logic."""
    if not MEM_CONN:
        return []

    grouper = get_grouper(group_key)
    if not grouper:
        raise HTTPException(status_code=404, detail=f"Grouper '{group_key}' not found")

    # Create Context
    ctx = QueryContext(
        db_conn=MEM_CONN,
        config=CONFIG,
        db_columns=VALID_ALBUM_COLUMNS
    )

    try:
        # Execute Plugin Logic
        # Grouper returns dict with "sql", "filter_target", "display_name"
        spec = grouper(ctx)
        
        sql = spec.get("sql")
        target = spec.get("filter_target")
        
        if not sql:
            return []
            
        cursor = MEM_CONN.execute(sql)
        rows = cursor.fetchall()
        
        # Transform into UI-friendly format
        return [
            {
                "label": str(row["value"]),
                "value": str(row["value"]),
                "count": row["count"],
                "filterTarget": target
            }
            for row in rows
        ]

    except Exception as e:
        print(f"Grouping Error ({group_key}): {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/library")
def get_library(
    filter_key: Optional[str] = Query(None, alias="filter"),
    filter_val: Optional[str] = Query(None, alias="val"),
    sort_key: str = Query("date_added", alias="sort"),
    sort_dir: str = "DESC"
):
    """
    Modular Query Factory.
    """
    if not MEM_CONN:
        return []

    # 1. Base Context
    ctx = QueryContext(
        db_conn=MEM_CONN,
        config=CONFIG,
        db_columns=VALID_ALBUM_COLUMNS,
        user_value=None,
        request_params={"filter": filter_key, "val": filter_val, "sort": sort_key, "dir": sort_dir}
    )

    # 2. Filter Logic
    where_clauses = []
    params = []

    if filter_key:
        filter_func = get_filter(filter_key)
        if filter_func:
            # Inject Value
            ctx.user_value = filter_val
            # Execute
            f_result = filter_func(ctx)
            
            if f_result and "where" in f_result:
                where_clauses.append(f_result["where"])
                params.extend(f_result.get("params", []))
        else:
            print(f"Warning: Unknown filter '{filter_key}'")

    where_sql = "WHERE " + " AND ".join(where_clauses) if where_clauses else ""

    # 3. Sort Logic
    sort_func = get_sorter(sort_key)
    if sort_func:
        ctx.user_value = sort_dir
        order_sql = sort_func(ctx)
    else:
        # Fallback
        order_sql = f"date_added {sort_dir}"

    # 4. Final Assembly
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
        {where_sql}
        ORDER BY {order_sql}
    """

    try:
        cursor = MEM_CONN.execute(sql, params)
        rows = cursor.fetchall()
        
        albums = []
        for row in rows:
            try:
                tracks_list = json.loads(row["tracks_json"])
            except (ValueError, TypeError):
                tracks_list = []

            albums.append({
                "id": row["id"],
                "title": row["ALBUM"],
                "artist": row["ALBUMARTIST"],
                "color": generate_color(row["id"]),
                "cover_path": row["cover_path"],
                "tracks": tracks_list
            })
            
        return albums

    except Exception as e:
        print(f"Query Error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

# (Asset and Play endpoints remain unchanged, omitted for brevity but assumed present)
@app.get("/api/assets/{album_id:path}/cover")
def get_album_cover(album_id: str):
    lib_root = get_library_root()
    if not lib_root or not MEM_CONN:
        raise HTTPException(status_code=404)
    try:
        cursor = MEM_CONN.execute("SELECT cover_path FROM albums WHERE id = ?", (album_id,))
        row = cursor.fetchone()
        if not row:
            cursor = MEM_CONN.execute("SELECT cover_path FROM tracks WHERE album_id = ? LIMIT 1", (album_id,))
            row = cursor.fetchone()
        if not row or not row["cover_path"] or row["cover_path"] == "default_cover.png":
            raise HTTPException(status_code=404)
        return FileResponse(lib_root / album_id / row["cover_path"])
    except:
        raise HTTPException(status_code=404)

@app.post("/api/play/{album_id:path}")
def play_album(album_id: str):
    # (Same implementation as before)
    return {"status": "ok"}

if __name__ == "__main__":
    # Ensure 'server.main:app' matches the module path
    uvicorn.run("server.main:app", host="127.0.0.1", port=8000, reload=True)
