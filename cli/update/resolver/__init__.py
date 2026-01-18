import sys
import importlib.util
from pathlib import Path
from . import tags
from . import helpers

__all__ = ["setup_registry", "find_resolver", "get_registered_keys"]

# The Registry: Separated by Scope and Type
_REGISTRY = {
    "ALBUM_TAGS": {},
    "ALBUM_HELPERS": {},
    "TRACK_TAGS": {},
    "TRACK_HELPERS": {}
}

_IS_INITIALIZED = False

def _register_module(module, allowlist=None):
    """
    Scans a module for functions matching the naming convention:
      resolve_album_tag_{NAME}
      resolve_album_helper_{name}
      resolve_track_tag_{NAME}
      resolve_track_helper_{name}
    
    allowlist: If provided (list of strings), only registers keys present in this list.
               The list contains keys (e.g. "is_cd", "GENRE"), not full function names.
    """
    
    # 1. Identify all candidates in module
    candidates = [k for k in dir(module) if k.startswith("resolve_")]
    
    for func_name in candidates:
        val = getattr(module, func_name)
        if not callable(val):
            continue

        # Parse Signature
        # resolve_{scope}_{type}_{key}
        parts = func_name.split("_", 3)
        if len(parts) < 4:
            continue
            
        scope = parts[1] # "album" or "track"
        kind = parts[2]  # "tag" or "helper"
        key_raw = parts[3] # "title", "bitrate", etc.
        
        # Determine Registry Bucket and Final Key format
        bucket = None
        final_key = None
        
        if scope == "album" and kind == "tag":
            bucket = "ALBUM_TAGS"
            final_key = key_raw.upper()
        elif scope == "album" and kind == "helper":
            bucket = "ALBUM_HELPERS"
            final_key = key_raw
        elif scope == "track" and kind == "tag":
            bucket = "TRACK_TAGS"
            final_key = key_raw.upper()
        elif scope == "track" and kind == "helper":
            bucket = "TRACK_HELPERS"
            final_key = key_raw
        else:
            continue

        # Check Allowlist (if active)
        if allowlist is not None:
            if final_key not in allowlist:
                continue

        # Register
        _REGISTRY[bucket][final_key] = val

def setup_registry(extensions_path_str: str = None, config_extensions: dict = None):
    """
    Initializes the registry with Standard Lib and Extensions.
    
    config_extensions: The flattened dictionary from config.toml [compiler.extensions]
                       Format: { "filename": ["key1", "key2"] }
    """
    global _IS_INITIALIZED
    if _IS_INITIALIZED:
        return

    # 1. Load Standard Library (Implicitly trusted, no allowlist)
    _register_module(tags)
    _register_module(helpers)
    
    # 2. Load Extensions
    if extensions_path_str and config_extensions:
        ext_root = Path(extensions_path_str).expanduser().resolve()
        
        for filename, keys in config_extensions.items():
            py_path = ext_root / f"{filename}.py"
            
            if not py_path.exists():
                print(f"Warning: Extension file not found: {py_path}")
                continue

            try:
                spec = importlib.util.spec_from_file_location(filename, py_path)
                if spec and spec.loader:
                    mod = importlib.util.module_from_spec(spec)
                    sys.modules[filename] = mod
                    spec.loader.exec_module(mod)
                    
                    # Register with explicit allowlist
                    # We ensure keys are stringified just in case
                    allowed_keys = [str(k) for k in keys]
                    _register_module(mod, allowlist=allowed_keys)
            except Exception as e:
                print(f"Critical Error loading extension '{filename}': {e}")
                sys.exit(1)

    _IS_INITIALIZED = True

def find_resolver(key: str, bucket_name: str):
    """
    Lookup function.
    bucket_name: "ALBUM_TAGS", "ALBUM_HELPERS", "TRACK_TAGS", "TRACK_HELPERS"
    """
    return _REGISTRY.get(bucket_name, {}).get(key)

def get_registered_keys():
    """
    Returns the 4 lists of keys available for calculation.
    """
    return (
        list(_REGISTRY["ALBUM_TAGS"].keys()),
        list(_REGISTRY["ALBUM_HELPERS"].keys()),
        list(_REGISTRY["TRACK_TAGS"].keys()),
        list(_REGISTRY["TRACK_HELPERS"].keys())
    )
