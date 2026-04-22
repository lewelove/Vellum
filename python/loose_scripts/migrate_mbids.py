import sys
import shutil
from pathlib import Path

def process_metadata_path(raw_path):
    clean_path = raw_path.strip().strip('"').strip("'")
    if not clean_path:
        return
        
    path = Path(clean_path).resolve()
    
    if not path.exists():
        print(f"IO_ERROR: Targeted manifest not found at \"{path}\"", file=sys.stderr)
        return

    mbid_path = path.parent / "mbid.toml"
    
    try:
        shutil.copy2(path, mbid_path)
    except Exception as e:
        print(f"COPY_ERROR: Failed to create mbid.toml for \"{path.parent.name}\": {e}", file=sys.stderr)
        return

    try:
        with open(path, "r", encoding="utf-8") as f:
            lines = f.readlines()
        with open(path, "w", encoding="utf-8") as f:
            for line in lines:
                if not line.lstrip().startswith("MUSICBRAINZ_"):
                    f.write(line)
    except Exception as e:
        print(f"WRITE_ERROR: Failed to strip IDs from metadata.toml at \"{path}\": {e}", file=sys.stderr)

    allowed_prefixes = (
        "[album]",
        "ALBUMARTIST",
        "ALBUM",
        "DATE",
        "[[tracks]]",
        "TRACKNUMBER",
        "TITLE",
        "MUSICBRAINZ_"
    )

    try:
        with open(mbid_path, "r", encoding="utf-8") as f:
            lines = f.readlines()
        with open(mbid_path, "w", encoding="utf-8") as f:
            for line in lines:
                stripped = line.strip()
                if not stripped:
                    f.write(line)
                    continue
                if any(stripped.startswith(p) for p in allowed_prefixes):
                    f.write(line)
    except Exception as e:
        print(f"WRITE_ERROR: Failed to filter mbid.toml at \"{mbid_path}\": {e}", file=sys.stderr)

    print(f"SUCCESS: Migrated \"{path.parent.name}\"")

def main():
    if len(sys.argv) > 1:
        for arg in sys.argv[1:]:
            process_metadata_path(arg)
    else:
        for line in sys.stdin:
            process_metadata_path(line)

if __name__ == "__main__":
    main()
