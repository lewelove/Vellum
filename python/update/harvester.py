import json
import subprocess
from pathlib import Path
from typing import List, Union

def harvest_metadata(target_paths: Union[Path, List[Path]]):
    """
    Invokes the Rust vellum binary to harvest metadata for specific paths.
    Returns a dictionary mapping absolute file paths to harvested data.
    """
    if isinstance(target_paths, Path):
        target_paths = [target_paths]
        
    if not target_paths:
        return {}

    try:
        cmd = ["vellum", "harvest"] + [str(p) for p in target_paths]
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True
        )
        
        harvested_map = {}
        for line in result.stdout.strip().split("\n"):
            if not line:
                continue
            try:
                item = json.loads(line)
                abs_path = str(Path(item["path"]).resolve())
                harvested_map[abs_path] = item
            except json.JSONDecodeError:
                continue
            
        return harvested_map
    except subprocess.CalledProcessError as e:
        print(f"Harvester Error: Failed to harvest metadata via Rust binary: {e.stderr}")
        return {}
    except Exception as e:
        print(f"Harvester Error: Unexpected error during harvesting: {e}")
        return {}
