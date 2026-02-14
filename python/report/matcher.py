import json
from pathlib import Path
from difflib import SequenceMatcher

def get_library_metadata(library_root):
    """Scans library for lock files and builds a lookup map."""
    lookup = {}
    lib_path = Path(library_root).expanduser().resolve()
    
    for lock_path in lib_path.rglob("metadata.lock.json"):
        try:
            with open(lock_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                album = data.get("album", {})
                
                raw_artist = album.get("ALBUMARTIST", "Unknown Artist")
                raw_title = album.get("ALBUM", "Unknown Album")
                
                artist_key = raw_artist.lower()
                title_key = raw_title.lower()
                
                total_tracks = int(album.get("TOTALTRACKS", 1))
                
                lookup[(artist_key, title_key)] = {
                    "total_tracks": total_tracks,
                    "display_artist": raw_artist,
                    "display_album": raw_title
                }
        except (json.JSONDecodeError, ValueError, OSError):
            continue
    return lookup

def match_listens(lb_counts, lib_lookup, fuzzy_threshold=0.85, debug_threshold=0.7):
    """
    Matches scrobbles to library.
    1. Attempts strict normalization match.
    2. If fails, attempts fuzzy match.
    3. If score > fuzzy_threshold, treats as matched.
    4. If score > debug_threshold, adds to debug list.
    """
    matched = []
    unknown = []
    fuzzy_debug = []
    
    lib_entries = list(lib_lookup.values())

    for (artist, release), count in lb_counts.items():
        key = (artist.lower(), release.lower())
        
        # 1. Strict Match
        if key in lib_lookup:
            meta = lib_lookup[key]
            album_listens = round(count / meta["total_tracks"], 2)
            matched.append({
                "artist": meta["display_artist"],
                "album": meta["display_album"],
                "listens": album_listens,
                "suffix": ""
            })
            continue

        # 2. Fuzzy Match Logic
        lb_str = f"{artist} - {release}".lower()
        best_score = 0
        best_meta = None
        
        for meta in lib_entries:
            lib_str = f"{meta['display_artist']} - {meta['display_album']}".lower()
            score = SequenceMatcher(None, lb_str, lib_str).ratio()
            if score > best_score:
                best_score = score
                best_meta = meta
        
        # 3. High Confidence Auto-Match
        if best_score >= fuzzy_threshold:
            album_listens = round(count / best_meta["total_tracks"], 2)
            matched.append({
                "artist": best_meta["display_artist"],
                "album": best_meta["display_album"],
                "listens": album_listens,
                "suffix": f" (fuzzy = {round(best_score, 2)})"
            })
        else:
            unknown.append({
                "artist": artist,
                "album": release,
                "listens": count
            })
            # 4. Low Confidence Debug Info
            if best_score >= debug_threshold:
                fuzzy_debug.append({
                    "lb_tuple": f"{artist} - {release}",
                    "lib_tuple": f"{best_meta['display_artist']} - {best_meta['display_album']}",
                    "score": round(best_score, 2)
                })
            
    return matched, unknown, fuzzy_debug
