import orjson
from difflib import SequenceMatcher
from pathlib import Path

# --- CONFIGURATION ---
TARGET_DIRECTORY = "/run/media/lewelove/1000xhome/backup-everything/FB2K/Library Historyfied!/"
THRESHOLD = 0.95  # Entries with a score LESS than this will be printed
METADATA_FILE = "metadata.lock.json"
# ---------------------

def get_similarity(a, b):
    """Calculates a similarity score between 0 and 1."""
    return SequenceMatcher(None, str(a), str(b)).ratio()

def main():
    root_path = Path(TARGET_DIRECTORY)
    
    if not root_path.exists():
        print(f"Error: Path '{TARGET_DIRECTORY}' does not exist.")
        return

    mismatches = []

    # Recursively find all metadata.lock.json files
    print(f"Scanning for {METADATA_FILE} files...")
    
    for metadata_path in root_path.rglob(METADATA_FILE):
        try:
            # orjson parsing
            with open(metadata_path, "rb") as f:
                data = orjson.loads(f.read())
            
            tracks = data.get("tracks", [])
            album_name = data.get("album", {}).get("ALBUM", "Unknown Album")
            
            for track in tracks:
                track_num = track.get("TRACKNUMBER", "")
                title = track.get("TITLE", "")
                encoding = track.get("encoding", "").lower()
                track_path = track.get("track_path", "")

                # Construct: {TRACKNUMBER - TITLE.encoding}
                # Example: "10 - Psycho Killer.flac"
                extension = f".{encoding}" if encoding else ""
                constructed = f"{track_num} - {title}{extension}"

                score = get_similarity(constructed, track_path)

                if score < THRESHOLD:
                    mismatches.append({
                        "score": round(score, 4),
                        "constructed": constructed,
                        "actual": track_path,
                        "album": album_name,
                        "location": metadata_path
                    })
        
        except Exception as e:
            print(f"Error processing {metadata_path}: {e}")

    # Sort by score DESCENDING (highest score first)
    mismatches.sort(key=lambda x: x["score"], reverse=True)

    # Output Results
    if not mismatches:
        print(f"No tracks found with a similarity score below {THRESHOLD}.")
        return

    print(f"\n{'SCORE':<8} | {'ALBUM':<30} | {'CONSTRUCTED vs ACTUAL'}")
    print("=" * 125)

    for item in mismatches:
        print(f"{item['score']:<8} | {str(item['album'])[:30]:<30} | EXPECTED: {item['constructed']}")
        print(f"{' ' * 8} | {' ' * 30} | ACTUAL:   {item['actual']}")
        print(f"{' ' * 8} | {' ' * 30} | FILE:     {item['location']}")
        print("-" * 125)

    print(f"\nFound {len(mismatches)} tracks below threshold {THRESHOLD}.")

if __name__ == "__main__":
    main()
