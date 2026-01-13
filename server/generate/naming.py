import re
from typing import List

def sanitize_slug(text: str, replacement: str = "_") -> str:
    """
    Removes illegal filename characters AND replaces whitespace.
    """
    text = re.sub(r'[\\/*?:"<>|]', replacement, text)
    
    text = re.sub(r'\s+', replacement, text)
    
    return text

def generate_filename(
    components: List[str], 
    separator: str, 
    replacement: str = "_"
) -> str:
    """
    Generates a unique filename slug from a list of components.
    
    1. Filters out empty components.
    2. Joins components with the raw separator.
    3. Sanitizes the entire resulting string at the end.
       (This ensures that if the user's separator contains illegal chars
        or spaces, they are safely normalized).
    """
    valid_parts = [str(c) for c in components if c]
    
    raw_slug = separator.join(valid_parts)
    
    return sanitize_slug(raw_slug, replacement)
