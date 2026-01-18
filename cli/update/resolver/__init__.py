import sys
import importlib.util
from pathlib import Path
from . import tags
from . import helpers

__all__ = ["setup_registry", "find_resolver"]

# The Registry: { "tag_name": func, "helper_name": func }
_RESOLVER_REGISTRY = {}
_IS_INITIALIZED = False

def _register_module(module, allowed_keys=None):
    """
    Scans a module for resolve_tag_* and resolve_helper_* functions.
    If allowed_keys is provided (list of strings), only registers those specific logic points.
    If allowed_keys is None, it registers nothing (safeguard).
    """
    if allowed_keys is None:
        # Standard library modules (tags.py, helpers.py) pass None to skip the check
        # and register everything found. We can handle this logic or just pass dir(module)
        # for std lib.
        keys_to_scan = [
            k for k in dir(module) 
            if k.startswith("resolve_tag_") or k.startswith("resolve_helper_")
        ]
    else:
        # User extension: strictly map keys to expected function names
        keys_to_scan = []
        for key in allowed_keys:
            if key.isupper():
                # TAG
                fname = f"resolve_tag_{key.lower()}"
                if not hasattr(module, fname):
                    raise ValueError(f"Extension Error: Module '{module.__name__}' does not define '{fname}' for key '{key}'")
                keys_to_scan.append(fname)
            else:
                # helper
                fname = f"resolve_helper_{key}"
                if not hasattr(module, fname):
                    raise ValueError(f"Extension Error: Module '{module.__name__}' does not define '{fname}' for key '{key}'")
                keys_to_scan.append(fname)

    for attr in keys_to_scan:
        val = getattr(module, attr)
        if not callable(val):
            continue
            
        if attr.startswith("resolve_tag_"):
            # resolve_tag_albumartist -> ALBUMARTIST
            key = attr[12:].upper()
            _RESOLVER_REGISTRY[f"TAG_{key}"] = val
            
        elif attr.startswith("resolve_helper_"):
            # resolve_helper_bitrate -> bitrate
            key = attr[15:]
            _RESOLVER_REGISTRY[f"HELPER_{key}"] = val

def setup_registry(extensions_path_str: str = None, config_registry: dict = None):
    """
    Initializes the registry with Standard Lib and Extensions.
    This should be called once by the compiler.
    
    config_registry: The dictionary from config.toml [compiler.extensions]
                     Expected format: { "album": { "file": ["key"] }, "tracks": { "file": ["key"] } }
    """
    global _IS_INITIALIZED
    if _IS_INITIALIZED:
        return

    # 1. Load Standard Library (Implicitly trusted)
    _register_module(tags, allowed_keys=None)
    _register_module(helpers, allowed_keys=None)
    
    # 2. Load Extensions (Explicitly trusted)
    if extensions_path_str and config_registry:
        ext_root = Path(extensions_path_str).expanduser().resolve()
        
        # Merge album and tracks config to load module once per file
        # Key = filename, Value = set of keys to import
        files_to_load = {}
        
        for scope in ["album", "tracks"]:
            scope_dict = config_registry.get(scope, {})
            for filename, keys in scope_dict.items():
                if filename not in files_to_load:
                    files_to_load[filename] = set()
                files_to_load[filename].update(keys)

        for filename, keys in files_to_load.items():
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
                    
                    # Register only the explicitly requested keys
                    _register_module(mod, allowed_keys=list(keys))
            except Exception as e:
                # If an explicit extension fails, it's a critical error
                print(f"Critical Error loading extension '{filename}': {e}")
                sys.exit(1)

    _IS_INITIALIZED = True

def find_resolver(key: str, category: str):
    """
    Lookup function.
    category: "TAG" (uppercase key) or "HELPER" (lowercase key)
    """
    if category == "TAG":
        lookup = f"TAG_{key.upper()}"
    else:
        lookup = f"HELPER_{key}"
        
    return _RESOLVER_REGISTRY.get(lookup)
