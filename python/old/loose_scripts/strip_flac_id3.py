import sys
from pathlib import Path
from mutagen.id3 import ID3
from mutagen.id3 import ID3NoHeaderError

def clean_flac_id3(directory):
    count = 0
    print(f"Scanning {directory} for FLAC files with ID3 tags...")
    for path in Path(directory).rglob("*.flac"):
        try:
            # Try to load an ID3 header. If it exists, delete it.
            id3_tags = ID3(path)
            id3_tags.delete(path)
            print(f"Removed ID3v2 tag from: {path.name}")
            count += 1
        except ID3NoHeaderError:
            # This is exactly what we want (no ID3 tag present)
            pass
        except Exception as e:
            print(f"Error processing {path.name}: {e}")
            
    print(f"\nDone! Cleaned {count} files.")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python strip_flac_id3.py <path_to_library>")
        sys.exit(1)
    clean_flac_id3(sys.argv[1])
