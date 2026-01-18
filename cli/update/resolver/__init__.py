import sys
import importlib.util
from pathlib import Path
from . import tags
from . import helpers

__all__ = ["setup_registry", "find_resolver"]

# The Registry: { "tag_name": func, "helper_name": func }
_RESOLVER_REGISTRY = {}
_IS_INITIALIZED = False

def _register_module(module):
    """
    Scans a module for resolve_tag_* and resolve_helper_* functions
    and registers them.
    """
    for attr in dir(module):
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

def setup_registry(extensions_path_str: str = None):
    """
    Initializes the registry with Standard Lib and Extensions.
    This should be called once by the compiler.
    """
    global _IS_INITIALIZED
    if _IS_INITIALIZED:
        return

    # 1. Load Standard Library (Last one imported wins, but these are base)
    _register_module(tags)
    _register_module(helpers)
    
    # 2. Load Extensions
    if extensions_path_str:
        ext_path = Path(extensions_path_str).expanduser().resolve()
        if ext_path.exists() and ext_path.is_dir():
            for py_file in ext_path.glob("*.py"):
                try:
                    spec = importlib.util.spec_from_file_location(py_file.stem, py_file)
                    if spec and spec.loader:
                        mod = importlib.util.module_from_spec(spec)
                        sys.modules[spec.name] = mod
                        spec.loader.exec_module(mod)
                        _register_module(mod)
                except Exception as e:
                    print(f"Warning: Failed to load extension {py_file}: {e}")

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
