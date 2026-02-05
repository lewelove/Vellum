import json
import subprocess
from pathlib import Path

def harvest_metadata(target_path: Path):
    """
    Invokes the Rust vellum binary to harvest metadata for a path.
    Returns a dictionary mapping absolute file paths to harvested data.
    """
    try:
        result = subprocess.run(
            ["vellum", "harvest", str(target_path)],
            capture_output=True,
            text=True,
            check=True
        )
        
        harvested_map = {}
        for line in result.stdout.strip().split("\n"):
            if not line:
                continue
            item = json.loads(line)
            abs_path = str(Path(item["path"]).resolve())
            harvested_map[abs_path] = item
            
        return harvested_map
    except subprocess.CalledProcessError as e:
        print(f"Harvester Error: Failed to harvest metadata via Rust binary: {e.stderr}")
        return {}
    except Exception as e:
        print(f"Harvester Error: Unexpected error during harvesting: {e}")
        return {}
