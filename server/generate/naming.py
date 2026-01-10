import re
from typing import Optional

def sanitize_slug(text: str, replacement: str = "_") -> str:
    """
    Removes illegal filename characters AND replaces whitespace.
    """
    # 1. Replace illegal chars: \ / * ? : " < > |
    text = re.sub(r'[\\/*?:"<>|]', replacement, text)
    
    # 2. Replace all whitespace (space, tab, newline)
    text = re.sub(r'\s+', replacement, text)
    
    return text

def generate_filename(
    pattern: str, 
    artist: str, 
    album: str, 
    custom_id: Optional[str], 
    replacement: str = "_"
) -> str:
    """
    Generates a unique, sanitized filename slug.
    Injects CUSTOM_ID if present to prevent collisions.
    """
    # Basic Interpolation
    base = pattern.replace("{artist}", artist).replace("{album}", album)
    
    # ID Injection
    if custom_id:
        base = f"{base}_{custom_id}"
        
    return sanitize_slug(base, replacement)
