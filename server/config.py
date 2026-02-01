import tomllib
import json
import os
from pathlib import Path

CONFIG = {}
LIBRARY_ROOT = None
THUMBNAIL_ROOT = None
STATE_DIR = Path("~/.eluxum").expanduser().resolve()
STATE_FILE = STATE_DIR / "state.json"

UI_STATE = {
    "activeTab": "home",
    "sortKey": "default",
    "groupKey": "genre",
    "filter": {"key": None, "val": None}
}

def load_config():
    global CONFIG, LIBRARY_ROOT, THUMBNAIL_ROOT
    config_path = Path("config.toml")
    if not config_path.exists():
        CONFIG = {}
    else:
        with open(config_path, "rb") as f:
            CONFIG = tomllib.load(f)
    
    root_str = CONFIG.get("storage", {}).get("library_root")
    thumb_str = CONFIG.get("storage", {}).get("thumbnail_cache_folder")
    
    if root_str: 
        LIBRARY_ROOT = Path(root_str).expanduser().resolve()
    if thumb_str: 
        THUMBNAIL_ROOT = Path(thumb_str).expanduser().resolve()

def load_ui_state():
    global UI_STATE
    if STATE_FILE.exists():
        try:
            with open(STATE_FILE, "rb") as f:
                saved = json.loads(f.read())
                UI_STATE.update(saved)
        except Exception as e:
            print(f"Warning: Could not load state.json: {e}")

def save_ui_state():
    try:
        STATE_DIR.mkdir(parents=True, exist_ok=True)
        tmp_file = STATE_FILE.with_suffix(".tmp")
        with open(tmp_file, "w", encoding="utf-8") as f:
            json.dump(UI_STATE, f, indent=2)
        os.replace(tmp_file, STATE_FILE)
    except Exception as e:
        print(f"Error saving state: {e}")
