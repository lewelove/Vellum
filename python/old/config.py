import tomllib
import os
from pathlib import Path

def resolve_config_path():
    env_path = os.environ.get("VELLUM_CONFIG_PATH")
    if env_path:
        p = Path(env_path).expanduser()
        if p.exists():
            return p

    home_config = Path("~/.config/vellum/config.toml").expanduser()
    if home_config.exists():
        return home_config

    curr = Path.cwd()
    for _ in range(5):
        for target in [curr / "config" / "config.toml", curr / "config.toml"]:
            if target.exists():
                return target
        if curr.parent == curr:
            break
        curr = curr.parent

    return None

def deep_merge(base, overlay):
    for k, v in overlay.items():
        if k == "import":
            continue
        if isinstance(v, dict) and k in base and isinstance(base[k], dict):
            deep_merge(base[k], v)
        else:
            base[k] = v
    return base

def load_config():
    cfg_path = resolve_config_path()
    if not cfg_path:
        raise FileNotFoundError("Could not locate vellum config.toml")

    visited = set()

    def load_recursive(path):
        canon = path.resolve()
        if canon in visited:
            return {}
        visited.add(canon)

        if not path.exists():
            return {}

        with open(path, "rb") as f:
            data = tomllib.load(f)

        merged = {}
        if "import" in data:
            imports = data["import"]
            if isinstance(imports, str):
                imports = [imports]
            
            base_dir = path.parent
            for rel in imports:
                imported_data = load_recursive(base_dir / rel)
                deep_merge(merged, imported_data)

        deep_merge(merged, data)
        return merged

    return load_recursive(cfg_path)
