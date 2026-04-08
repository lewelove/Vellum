import os
import sys
import json
import re
import argparse
from pathlib import Path
import lyricsgenius

def clean_genius_lyrics(lyrics, title):
    if not lyrics:
        return ""
    
    lines = lyrics.split("\n")
    if lines and "Contributors" in lines[0]:
        lines.pop(0)
    
    cleaned = "\n".join(lines)
    cleaned = re.sub(r"[0-9]*Embed$", "", cleaned)
    cleaned = cleaned.strip()
    
    return cleaned

def sanitize_filename(name):
    return re.sub(r'[<>:"/\\|?*]', '_', name)

def fetch_album_lyrics(album_root_path, access_token):
    root = Path(album_root_path).expanduser().resolve()
    lock_file = root / "metadata.lock.json"
    
    if not lock_file.exists():
        print(f"Error: metadata.lock.json not found in {root}")
        return

    with open(lock_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    album_artist = data.get("album", {}).get("ALBUMARTIST")
    total_discs = int(data.get("album", {}).get("info", {}).get("total_discs", 1))
    tracks = data.get("tracks", [])

    if not album_artist or not tracks:
        print("Error: Invalid metadata structure in lock file.")
        return

    genius = lyricsgenius.Genius(access_token)
    genius.verbose = False
    genius.remove_section_headers = False

    lyrics_dir = root / "Lyrics"
    lyrics_dir.mkdir(exist_ok=True)

    print(f"Fetching lyrics for: {album_artist} - {data.get('album', {}).get('ALBUM')}")

    for track in tracks:
        title = track.get("TITLE")
        track_num = str(track.get("TRACKNUMBER", "0")).zfill(2)
        disc_num = str(track.get("DISCNUMBER", "1"))
        
        if not title:
            continue

        safe_title = sanitize_filename(title)

        if total_discs > 1:
            filename = f"{disc_num}.{track_num} - {safe_title}.txt"
        else:
            filename = f"{track_num} - {safe_title}.txt"
            
        dest_path = lyrics_dir / filename

        if dest_path.exists():
            print(f"  Skipping: {title} (File exists)")
            continue

        try:
            song = genius.search_song(title, album_artist)
            if song:
                cleaned_text = clean_genius_lyrics(song.lyrics, title)
                with open(dest_path, "w", encoding="utf-8") as lf:
                    lf.write(cleaned_text)
                print(f"  Saved: {title}")
            else:
                print(f"  Not found: {title}")
        except Exception as e:
            print(f"  Error fetching {title}: {e}")

def main():
    parser = argparse.ArgumentParser(description="Fetch lyrics for a Vellum album using Genius API")
    parser.add_argument(
        "path",
        help="Path to the album root directory containing metadata.lock.json"
    )
    parser.add_argument(
        "--token",
        help="Genius API Access Token (or set GENIUS_ACCESS_TOKEN env var)"
    )

    args = parser.parse_args()

    token = args.token or os.environ.get("GENIUS_ACCESS_TOKEN")
    if not token:
        print("Error: Genius Access Token is required via --token or GENIUS_ACCESS_TOKEN environment variable.")
        sys.exit(1)

    fetch_album_lyrics(args.path, token)

if __name__ == "__main__":
    main()
