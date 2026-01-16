import re
from pathlib import Path

def natural_sort_key(s):
    return [int(text) if text.isdigit() else text.lower() for text in re.split(r'(\d+)', str(s))]

def _parse_int(val) -> int:
    """
    Safely parses integer values from strings like '1', '01', '1/12'.
    Defaults to 0 on failure.
    """
    if val is None:
        return 0
    s = str(val).strip()
    if "/" in s:
        s = s.split("/")[0]
    if s.isdigit():
        return int(s)
    return 0

def scan_physical_spine(album_root: Path, supported_exts: list) -> list:
    """
    PHASE 1: THE SPINE
    Returns list of relative paths sorted naturally.
    """
    files = []
    for ext in supported_exts:
        files.extend(album_root.rglob(f"*{ext}"))
    
    files = [f for f in files if not f.name.startswith('.')]
    rel_files = [p.relative_to(album_root) for p in files]
    rel_files.sort(key=lambda p: natural_sort_key(str(p)))
    
    return rel_files

def zip_tracks(inflated_tracks: list, physical_files: list) -> list:
    """
    PHASE 4: THE ZIP
    Matches tracks to files based on DISCNUMBER and TRACKNUMBER.
    
    COMPLIANCE LOGIC:
    - Sorts tracks by Disc and Track Number.
    - Calculates the 'delta' between the current track and the previous track.
    - If a numeric gap exists (e.g. Track 2 -> Track 4), it skips the corresponding 
      amount of files in the physical list.
    - Resets the track counter when the Disc Number changes, assuming sequential 
      mapping for the start of a new disc.
    """
    
    # 1. Prepare sortable wrappers for the tracks
    # We maintain a reference to the original dictionary to modify it in-place.
    wrapped_tracks = []
    for t in inflated_tracks:
        d = _parse_int(t.get("DISCNUMBER", "1"))
        n = _parse_int(t.get("TRACKNUMBER", "0"))
        wrapped_tracks.append({
            "d": d,
            "n": n,
            "ref": t
        })

    # 2. Sort tracks to ensure strictly linear processing
    wrapped_tracks.sort(key=lambda x: (x["d"], x["n"]))

    # 3. Zip with Delta Logic
    file_cursor = 0
    
    # State tracking
    last_disc = -1
    last_track_num = 0

    for w_track in wrapped_tracks:
        current_disc = w_track["d"]
        current_track_num = w_track["n"]

        # Detect Disc Change
        if current_disc != last_disc:
            last_disc = current_disc
            # Reset track counter for the new disc context
            last_track_num = 0
        
        # Calculate Gap (Delta)
        # Example: Prev=2, Curr=4. Delta=2. We need to skip 1 file (File for Trk 3).
        # Normal:  Prev=1, Curr=2. Delta=1. Skip 0.
        delta = max(1, current_track_num - last_track_num)
        
        # Advance cursor to skip missing files
        skip_count = delta - 1
        file_cursor += skip_count

        # Assign File
        if file_cursor < len(physical_files):
            w_track["ref"]["track_path"] = str(physical_files[file_cursor])
        else:
            w_track["ref"]["track_path"] = ""

        # Consume the file we just assigned
        file_cursor += 1
        
        # Update State
        last_track_num = current_track_num

    return inflated_tracks
