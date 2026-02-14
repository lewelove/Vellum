import argparse
import tomllib
from pathlib import Path

from .lb_parser import parse_listenbrainz_export
from .matcher import get_library_metadata, match_listens
from .formatter import generate_report_text

def run_report():
    parser = argparse.ArgumentParser(description="Vellum Report Generator")
    parser.add_argument("--year", type=int, help="Filter by year (YYYY)")
    parser.add_argument("--listenbrainz", required=True, help="Path to ListenBrainz export zip")
    parser.add_argument("--total-album-listens-min", type=float, default=3.0)
    parser.add_argument("--total-track-listens-min", type=int, default=3)
    
    args = parser.parse_args()
    
    config_path = Path("config.toml")
    if not config_path.exists():
        print("Error: config.toml not found.")
        return

    with open(config_path, "rb") as f:
        config = tomllib.load(f)
    
    lib_root = config["storage"]["library_root"]
    
    print(f"Parsing {args.listenbrainz}...")
    lb_counts = parse_listenbrainz_export(args.listenbrainz, args.year)
    
    print("Scanning library metadata...")
    lib_lookup = get_library_metadata(lib_root)
    
    print("Processing and matching listens...")
    matched, unknown, fuzzy_debug = match_listens(lb_counts, lib_lookup)
    
    report = generate_report_text(
        args.year, 
        matched, 
        unknown, 
        fuzzy_debug,
        args.total_album_listens_min, 
        args.total_track_listens_min
    )
    
    print("\n" + report + "\n")
