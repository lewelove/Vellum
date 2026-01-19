from dataclasses import dataclass
from typing import Any, Dict, List, Optional
import sqlite3

@dataclass
class QueryContext:
    db_conn: sqlite3.Connection
    config: Dict[str, Any]
    db_columns: set
    user_value: Optional[Any] = None
    request_params: Optional[Dict[str, str]] = None
