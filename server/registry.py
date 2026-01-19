import importlib
import pkgutil
import inspect
from pathlib import Path
from types import ModuleType

# Buckets for the three types of logic
_FILTERS = {}
_SORTERS = {}
_GROUPERS = {}

def _register_module_functions(module: ModuleType):
    """
    Scans a module for functions matching the naming convention:
      filter_{name}
      sort_{name}
      group_{name}
    """
    for name, obj in inspect.getmembers(module, inspect.isfunction):
        if name.startswith("filter_"):
            key = name[7:] # strip 'filter_'
            _FILTERS[key] = obj
        elif name.startswith("sort_"):
            key = name[5:] # strip 'sort_'
            _SORTERS[key] = obj
        elif name.startswith("group_"):
            key = name[6:] # strip 'group_'
            _GROUPERS[key] = obj

def load_plugins(features_path: str = "server/features"):
    """
    Recursively scans the features directory and imports all modules,
    triggering the registration logic.
    """
    # Ensure standard library is loaded first
    base_path = Path(features_path).resolve()
    
    # We walk the package to find all sub-packages (filtering, sorting, grouping)
    # and their modules (std.py, custom.py)
    search_path = [str(base_path)]
    
    # Import the base package first to ensure path correctness
    base_pkg = "server.features"
    
    for finder, name, ispkg in pkgutil.walk_packages(search_path, prefix=base_pkg + "."):
        try:
            module = importlib.import_module(name)
            _register_module_functions(module)
        except Exception as e:
            print(f"Error loading plugin {name}: {e}")

def get_filter(key: str):
    return _FILTERS.get(key)

def get_sorter(key: str):
    return _SORTERS.get(key)

def get_grouper(key: str):
    return _GROUPERS.get(key)

def get_capabilities():
    return {
        "filtering": list(_FILTERS.keys()),
        "sorting": list(_SORTERS.keys()),
        "grouping": list(_GROUPERS.keys())
    }
