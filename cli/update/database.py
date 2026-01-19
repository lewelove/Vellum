import sqlite3
import hashlib
import re
from pathlib import Path

class Database:
    def __init__(self, db_path: Path, config_path: Path):
        self.db_path = db_path.expanduser().resolve()
        self.config_path = config_path.resolve()
        self.conn = None
        self.known_columns = {"albums": set(), "tracks": set()}
        
        # Ensure directory exists
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        
        self._connect()
        self._check_config_integrity()

    def _connect(self):
        self.conn = sqlite3.connect(self.db_path)
        self.conn.row_factory = sqlite3.Row
        self.conn.execute("PRAGMA journal_mode=WAL;")
        self.conn.execute("PRAGMA foreign_keys=ON;")

    def _get_file_hash(self, path: Path) -> str:
        sha256 = hashlib.sha256()
        with open(path, "rb") as f:
            while chunk := f.read(8192):
                sha256.update(chunk)
        return sha256.hexdigest()

    def _check_config_integrity(self):
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS system (
                key TEXT PRIMARY KEY, 
                value TEXT
            )
        """)
        self.conn.commit()

        current_hash = self._get_file_hash(self.config_path)
        
        cursor = self.conn.execute("SELECT value FROM system WHERE key = 'config_hash'")
        row = cursor.fetchone()
        stored_hash = row["value"] if row else None

        if stored_hash != current_hash:
            print(f"Config change detected. Rebuilding database at {self.db_path}...")
            self.conn.execute("DROP TABLE IF EXISTS tracks")
            self.conn.execute("DROP TABLE IF EXISTS albums")
            
            self.conn.execute("INSERT OR REPLACE INTO system (key, value) VALUES ('config_hash', ?)", (current_hash,))
            self.conn.commit()
            self.known_columns = {"albums": set(), "tracks": set()}

    def _sanitize_col(self, key: str) -> str:
        if not key: return ""
        return re.sub(r'[^a-zA-Z0-9_]', '', str(key))

    def _ensure_columns(self, table_name: str, data_keys: set):
        valid_keys = {self._sanitize_col(k) for k in data_keys if k}
        valid_keys.discard("")

        schema_changed = False

        # 1. Initialize table if missing
        if table_name not in self.known_columns or not self.known_columns[table_name]:
            try:
                cursor = self.conn.execute(f"PRAGMA table_info({table_name})")
                existing = {row["name"] for row in cursor.fetchall()}
                
                if not existing:
                    # Create Table
                    if table_name == "albums":
                        base = ['"id" TEXT PRIMARY KEY', '"lock_hash" TEXT']
                        reserved = {"id", "lock_hash"}
                    elif table_name == "tracks":
                        base = ['"id" TEXT PRIMARY KEY', '"album_id" TEXT REFERENCES albums("id") ON DELETE CASCADE']
                        reserved = {"id", "album_id"}
                    else:
                        return

                    dynamic = [f'"{k}" TEXT' for k in valid_keys if k not in reserved]
                    full_schema = ", ".join(base + dynamic)
                    
                    try:
                        self.conn.execute(f"CREATE TABLE {table_name} ({full_schema})")
                        self.known_columns[table_name] = reserved | valid_keys
                        schema_changed = True
                    except sqlite3.OperationalError as e:
                        print(f"CRITICAL: Failed to create table {table_name}. Schema: {full_schema}")
                        raise e 
                else:
                    self.known_columns[table_name] = existing
            except sqlite3.OperationalError:
                return

        # 2. Alter table if new keys found
        if not schema_changed:
            current_cols = self.known_columns[table_name]
            reserved = {"id", "album_id", "lock_hash"}
            new_cols = valid_keys - current_cols - reserved

            for col in new_cols:
                try:
                    self.conn.execute(f'ALTER TABLE {table_name} ADD COLUMN "{col}" TEXT')
                    self.known_columns[table_name].add(col)
                    schema_changed = True
                except sqlite3.OperationalError as e:
                    print(f"Warning: Failed to add column '{col}' to {table_name}: {e}")
        
        if schema_changed:
            self.conn.commit()

    def sync_album(self, album_id: str, lock_data: dict, lock_hash: str):
        # 1. Check existing
        try:
            cursor = self.conn.execute("SELECT lock_hash FROM albums WHERE id = ?", (album_id,))
            row = cursor.fetchone()
            if row and row["lock_hash"] == lock_hash:
                return
        except sqlite3.OperationalError:
            pass

        # 2. Prepare Data (Inflation Logic)
        album_payload = lock_data.get("album", {})
        raw_tracks = lock_data.get("tracks", [])
        
        # INFLATION: Merge Album Data into every Track
        inflated_tracks = []
        for track in raw_tracks:
            # Start with a copy of Album data
            merged = album_payload.copy()
            # Overlay specific Track data (Compiler intent respected via overwriting)
            merged.update(track)
            inflated_tracks.append(merged)
        
        # 3. Schema Sync
        # Sync albums table using album keys
        self._ensure_columns("albums", set(album_payload.keys()))
        
        # Sync tracks table using INFLATED keys (Union of Album + Track keys)
        all_track_keys = set()
        for t in inflated_tracks:
            all_track_keys.update(t.keys())
        self._ensure_columns("tracks", all_track_keys)

        # 4. Insert
        try:
            self.conn.execute("DELETE FROM tracks WHERE album_id = ?", (album_id,))
            
            # Album Insert
            reserved_alb = {"id", "lock_hash"}
            a_raw_keys = [k for k in album_payload.keys() if self._sanitize_col(k) and self._sanitize_col(k) not in reserved_alb]
            a_clean_keys = [self._sanitize_col(k) for k in a_raw_keys]
            
            final_keys = ["id", "lock_hash"] + a_clean_keys
            final_vals = [album_id, lock_hash] + [str(album_payload[k]) for k in a_raw_keys]
            
            placeholders = ", ".join(["?"] * len(final_keys))
            columns = ", ".join([f'"{k}"' for k in final_keys])
            
            self.conn.execute(
                f"INSERT OR REPLACE INTO albums ({columns}) VALUES ({placeholders})", 
                final_vals
            )

            # Tracks Insert (Using Inflated Data)
            reserved_trk = {"id", "album_id"}
            for track in inflated_tracks:
                t_id = track.get("track_library_path")
                if not t_id: continue
                
                t_raw_keys = [k for k in track.keys() if self._sanitize_col(k) and self._sanitize_col(k) not in reserved_trk]
                t_clean_keys = [self._sanitize_col(k) for k in t_raw_keys]
                
                final_t_keys = ["id", "album_id"] + t_clean_keys
                final_t_vals = [t_id, album_id] + [str(track[k]) for k in t_raw_keys]
                
                t_placeholders = ", ".join(["?"] * len(final_t_keys))
                t_columns = ", ".join([f'"{k}"' for k in final_t_keys])

                self.conn.execute(
                    f"INSERT INTO tracks ({t_columns}) VALUES ({t_placeholders})",
                    final_t_vals
                )

            self.conn.commit()
            
        except Exception as e:
            print(f"Database Error syncing {album_id}: {e}")
            self.conn.rollback()

    def prune(self, active_album_ids: list):
        if not active_album_ids:
            return
        placeholders = ", ".join(["?"] * len(active_album_ids))
        self.conn.execute(
            f"DELETE FROM albums WHERE id NOT IN ({placeholders})", 
            active_album_ids
        )
        self.conn.commit()

    def close(self):
        if self.conn:
            self.conn.close()
