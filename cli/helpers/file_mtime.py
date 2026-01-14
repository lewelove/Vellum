import os
from pathlib import Path

def resolve(file_path: Path) -> int:
    """
    Returns the last modification time of the file as a Unix timestamp (integer).
    Used for cache invalidation/trust checks.
    """
    try:
        return int(os.path.getmtime(file_path))
    except OSError:
        return 0
