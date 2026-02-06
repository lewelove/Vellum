import asyncio
import concurrent.futures
import orjson
import hashlib
import json
from pathlib import Path, PurePosixPath
from . import config

class LibraryState:
    def __init__(self):
        self.albums = []  
        self.album_map = {}  
        self.track_map = {}  
        self.path_lookup = {} 

    def _normalize(self, path_str: str) -> str:
        if not path_str: return ""
        return path_str.lstrip('/')

    def _update_current_hash(self):
        if not config.LIBRARY_ROOT:
            return
        try:
            hash_val = hashlib.sha256(str(config.LIBRARY_ROOT).encode()).hexdigest()
            config.CURRENT_LIB_FILE.parent.mkdir(parents=True, exist_ok=True)
            with open(config.CURRENT_LIB_FILE, "w", encoding="utf-8") as f:
                json.dump({"hash": hash_val}, f)
        except Exception as e:
            print(f"Warning: Could not update current.json: {e}")

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
        if not config.LIBRARY_ROOT:
            print("Library Root not configured.")
            return

        print(f"Scanning library at {config.LIBRARY_ROOT}...")
        self._update_current_hash()
        
        loop = asyncio.get_running_loop()
        lock_files = list(config.LIBRARY_ROOT.rglob("metadata.lock.json"))
        
        with concurrent.futures.ThreadPoolExecutor() as pool:
            results = await loop.run_in_executor(None, lambda: list(pool.map(self._parse_lock_file, lock_files)))
            
        self.albums, self.album_map, self.track_map, self.path_lookup = [], {}, {}, {}
        for res in results:
            if not res: continue
            album_data, t_map, p_map = res
            self.albums.append(album_data)
            self.album_map[album_data["id"]] = album_data
            for t_id, t_rel in t_map.items():
                self.track_map[t_id] = str(config.LIBRARY_ROOT / album_data["id"] / t_rel)
            self.path_lookup.update(p_map)

        self.albums.sort(key=lambda x: x["id"])
        print(f"Live Lake Initialized: {len(self.albums)} albums. {len(self.path_lookup)} tracks.")

    def update_album(self, folder_path_str: str):
        if not config.LIBRARY_ROOT: return None
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
            self.track_map[t_id] = str(config.LIBRARY_ROOT / alb_id / t_rel)
        self.path_lookup.update(new_p_map)
        return new_album

STATE = LibraryState()
