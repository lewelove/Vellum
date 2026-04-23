import sys
import argparse
from pathlib import Path

def process_file(file_path):
    try:
        content = file_path.read_text(encoding="utf-8")
        lines = content.splitlines()
        
        new_lines = []
        changed = False
        
        for line in lines:
            stripped = line.strip()
            if stripped.startswith("[") and stripped.endswith("]"):
                changed = True
                continue
            new_lines.append(line)
            
        if changed:
            file_path.write_text("\n".join(new_lines), encoding="utf-8")
            return True
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
    
    return False

def main():
    parser = argparse.ArgumentParser(description="Clean bracketed headers from existing lyrics files")
    parser.add_argument("path", help="Library root path to scan")
    args = parser.parse_args()
    
    root = Path(args.path).expanduser().resolve()
    if not root.exists():
        print(f"Error: Path {root} does not exist")
        return

    print(f"Scanning for lyrics in: {root}")
    
    clean_count = 0
    total_found = 0
    
    for lyrics_file in root.rglob("Lyrics/*.txt"):
        total_found += 1
        if process_file(lyrics_file):
            print(f"Cleaned: {lyrics_file.relative_to(root)}")
            clean_count += 1
            
    print(f"\nFinished. Scanned {total_found} files, cleaned {clean_count} files.")

if __name__ == "__main__":
    main()
