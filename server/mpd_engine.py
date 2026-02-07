import asyncio
import orjson
from pathlib import Path
from mpd import MPDClient, ConnectionError
from . import config
from .library import STATE

async def monitor_loop():
    # Local import to avoid circular dependency with api.py (which imports play_album_logic)
    from .api import manager
    
    client = MPDClient()
    print("Starting MPD Monitor...")
    
    last_playlist_version = None
    cached_queue = []

    while True:
        try:
            try:
                client.ping()
            except (ConnectionError, OSError):
                client.connect("localhost", 6600)

            status = client.status()
            current_playlist_version = status.get("playlist")
            
            if current_playlist_version != last_playlist_version:
                cached_queue = client.playlistinfo()
                last_playlist_version = current_playlist_version
            
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
                "queue": cached_queue
            }
            await manager.broadcast_bytes(orjson.dumps(payload))
            await asyncio.sleep(1)
            
        except (ConnectionError, OSError):
            await asyncio.sleep(2)
        except asyncio.CancelledError:
            try: client.disconnect()
            except: pass
            break
        except Exception:
            await asyncio.sleep(1)

def _get_album_paths(album_id: str):
    album = STATE.album_map.get(album_id)
    if not album: 
        return []
    return [
        STATE.track_map[t["track_library_path"]] 
        for t in album.get("tracks", []) 
        if t.get("track_library_path") in STATE.track_map
    ]

def play_album_logic(album_id: str, offset: int = 0):
    paths = _get_album_paths(album_id)
    if not paths: 
        return False
        
    client = MPDClient()
    try:
        client.connect("localhost", 6600)
        client.clear()
        
        for p in paths:
            if config.LIBRARY_ROOT:
                rel_p = Path(p).relative_to(config.LIBRARY_ROOT)
                client.add(str(rel_p))
                
        client.play(offset)
        client.close()
        return True
    except Exception as e:
        print(f"MPD Error: {e}")
        return False

def enqueue_album_logic(album_id: str):
    paths = _get_album_paths(album_id)
    if not paths:
        return False

    client = MPDClient()
    try:
        client.connect("localhost", 6600)
        for p in paths:
            if config.LIBRARY_ROOT:
                rel_p = Path(p).relative_to(config.LIBRARY_ROOT)
                client.add(str(rel_p))
        client.close()
        return True
    except Exception as e:
        print(f"MPD Error: {e}")
        return False
