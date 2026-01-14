import os
from pathlib import Path

def resolve(file_path: Path) -> int:
    """
    Returns the size of the file in bytes.
    Used for cache invalidation/trust checks.
    """
    try:
        return os.path.getsize(file_path)
    except OSError:
        return 0
