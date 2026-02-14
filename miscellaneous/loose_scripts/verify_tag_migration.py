import argparse
import subprocess
import json
import sys
from pathlib import Path

# --- CONFIGURATION ---
# Format: (Old/Legacy Key, New/Standardized Key)
MIGRATION_PAIRS = [
    ("UNIXTIMEFOOBAR", "UNIX_ADDED_FOOBAR"),
    ("UNIXTIMEAPPLE", "UNIX_ADDED_APPLEMUSIC"),
    ("UNIXTIMEYOUTUBE", "UNIX_ADDED_YOUTUBE"),
    ("ORIGINALMONTHYEAR", "ORIGINAL_DATE"),
    ("ORIGINALYEAR", "ORIGINAL_YEAR"),
    ("ORIGINALYEARMONTH", "ORIGINAL_YYYY_MM"),
    ("ARTISTARTIST", "CUSTOM_ALBUMARTIST"),
    ("CUSTOMSTRING", "CUSTOM_STRING"),
    # ("", ""),
    # ("", ""),
    # ("", ""),
]
# ---------------------

def get_harvest_data(target_path: Path):
    """
    Invokes the 'vellum harvest' command to get raw JSON output 
    describing the current state of files on disk.
    """
    try:
        # Calls the vellum binary assumed to be in the PATH or alias
        result = subprocess.run(
            ["vellum", "harvest", str(target_path)],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip().split("\n")
    except subprocess.CalledProcessError as e:
        print(f"Error running harvester: {e.stderr}")
        sys.exit(1)
    except FileNotFoundError:
        print("Error: 'vellum' command not found. Ensure it is installed and in your PATH.")
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(
        description="Verify that legacy tag values have been correctly migrated to new keys."
    )
    parser.add_argument("path", help="Path to the directory to verify")
    args = parser.parse_args()

    target_path = Path(args.path).expanduser().resolve()
    if not target_path.exists():
        print(f"Error: Path does not exist: {target_path}")
        return

    print(f"Harvesting metadata from: {target_path} ...")
    lines = get_harvest_data(target_path)

    total_files = 0
    discrepancies = 0
    verified_migrations = 0

    print("\n--- Scanning for Discrepancies ---\n")

    for line in lines:
        if not line:
            continue
        
        try:
            data = json.loads(line)
        except json.JSONDecodeError:
            continue

        file_path = data.get("path", "Unknown Path")
        tags = data.get("tags", {})
        
        file_has_error = False

        for source_key, target_key in MIGRATION_PAIRS:
            source_val = tags.get(source_key)
            
            # If the source key doesn't exist, there is nothing to migrate/verify for this pair
            if not source_val:
                continue

            target_val = tags.get(target_key)

            # Check 1: Target key missing entirely
            if target_val is None:
                print(f"[MISSING TARGET] {file_path}")
                print(f"  Source: {source_key} = '{source_val}'")
                print(f"  Target: {target_key} is MISSING")
                file_has_error = True
                continue

            # Check 2: Value mismatch
            # Normalize strings to avoid whitespace false positives
            s_norm = str(source_val).strip()
            t_norm = str(target_val).strip()

            if s_norm != t_norm:
                print(f"[MISMATCH] {file_path}")
                print(f"  Source: {source_key} = '{s_norm}'")
                print(f"  Target: {target_key} = '{t_norm}'")
                file_has_error = True
            else:
                verified_migrations += 1

        if file_has_error:
            discrepancies += 1
        
        total_files += 1

    print("\n--- Verification Summary ---")
    print(f"Files Scanned:       {total_files}")
    print(f"Verified Pairs:      {verified_migrations}")
    print(f"Files with Errors:   {discrepancies}")

    if discrepancies == 0:
        print("\nSUCCESS: All detected legacy keys match their target keys.")
        print("It is safe to delete the source keys from these files.")
    else:
        print(f"\nFAILURE: Found {discrepancies} files with migration issues.")
        print("Do NOT delete source keys until these are resolved.")

if __name__ == "__main__":
    main()
