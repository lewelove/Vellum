import json
from pathlib import Path

# --- DEVELOPER BLUEPRINTS ---
# Edit these lists to change the visual grouping and order in metadata.lock.
# Format: ("Category Label", ["KEY1", "KEY2", ...])

ALBUM_LOCK_BLUEPRINT = [
    ("\n", [
        "ALBUM", 
        "ALBUMARTIST", 
        "DATE", 
        "GENRE", 
    ]),
    ("\n", [
        "COMMENT",
        # "unix_added",
    ]),
    ("\n", [
        "ORIGINAL_DATE",
        "ORIGINAL_YEAR",
        "ORIGINAL_YYYY_MM",
    ]),
    ("\n", [
        "COUNTRY",
        "LABEL",
        "CATALOGNUMBER",
        "RELEASE_DATE",
        "RELEASE_YEAR",
        "RELEASE_YYYY_MM",
    ]),
    ("\n", [
        "MEDIA",
        "DISCOGS_URL",
        "MUSICBRAINZ_URL",
    ]),
    ("\n", [
        "ACCURIPID",
        "CTDBID",
        "DISCID",
    ]),
    ("\n", [
        "date_added",
    ]),
    ("\n", [
        "UNIX_ADDED_LOCAL",
        "UNIX_ADDED_APPLEMUSIC",
        "UNIX_ADDED_YOUTUBE",
        "UNIX_ADDED_PRIMARY",
    ]),
    ("\n", [
        "CUSTOM_ID",
        "CUSTOM_ALBUMARTIST",
        "CUSTOM_STRING",
        "OLD_COMMENT",
    ]),
    ("\n", [
        "album_duration_time",
        "TOTALDISCS",
        "TOTALTRACKS",
    ]),
    ("\n", [
        "MUSICBRAINZ_ALBUMID",
        "MUSICBRAINZ_ALBUMARTISTID",
        "MUSICBRAINZ_RELEASEGROUPID",
    ]),
    ("\n", [
        "album_root_path",
        "cover_path",
        "unix_added",
    ]),
    ("\n", [
        "metadata_toml_mtime",
        "metadata_toml_hash",
        "cover_mtime",
        "cover_byte_size",
    ]),
]

TRACK_LOCK_BLUEPRINT = [
    ("", [
        "TITLE",
        "ARTIST",
        "track_path",
        "TRACKNUMBER",
        "DISCNUMBER",
        "track_duration_time",
        "encoding",
        "sample_rate",
        "bits_per_sample",
        "channels",
        "track_mtime",
        "track_size",
        "track_duration_in_ms",
        "track_duration_in_samples",
        "lyrics_path",
        "lyrics_path_absolute",
        "track_path_absolute",
    ]),
    ("", [
        "MUSICBRAINZ_TRACKID",
        "MUSICBRAINZ_RELEASETRACKID",
        "MUSICBRAINZ_ARTISTID",
    ])
]

def format_val(value):
    """Ensures values are TOML-compliant (JSON-style strings/numbers)."""
    return json.dumps(value, ensure_ascii=False)

def structure_lock_layout(data: dict, blueprint: list) -> list:
    """
    Transforms a data dictionary into a list of formatted TOML lines
    based on the provided blueprint.
    """
    lines = []
    consumed_keys = set()

    for label, keys in blueprint:
        # Only render the category if at least one key exists in the data
        section_keys = [k for k in keys if k in data]
        if not section_keys:
            continue
        
        if label:
            lines.append(f"{label}")
            
        for k in section_keys:
            lines.append(f"{k} = {format_val(data[k])}")
            consumed_keys.add(k)
        
    lines.append("") # Group spacer

    # Appendix: Catch-all for any compiled keys not mentioned in the blueprint
    appendix_keys = sorted([k for k in data.keys() if k not in consumed_keys])
    if appendix_keys:
        lines.append("")
        for k in appendix_keys:
            lines.append(f"{k} = {format_val(data[k])}")
        lines.append("")

    return lines

def write_lock(album_root: Path, album_data: dict, tracks_data: list):
    """
    Main entry point for writing the compiled metadata.lock file.
    """
    final_output = []

    # 1. Render Album Section
    final_output.append("[album]")
    final_output.extend(structure_lock_layout(album_data, ALBUM_LOCK_BLUEPRINT))

    # 2. Render Tracks Sections
    for track in tracks_data:
        final_output.append("[[tracks]]")
        final_output.extend(structure_lock_layout(track, TRACK_LOCK_BLUEPRINT))

    # 3. Write to file
    lock_path = album_root / "metadata.lock"
    # Filter out redundant double-spacers but keep structure
    cleaned_content = "\n".join(final_output).replace("\n\n\n", "\n\n")
    
    with open(lock_path, "w", encoding="utf-8") as f:
        f.write(cleaned_content.strip() + "\n")
