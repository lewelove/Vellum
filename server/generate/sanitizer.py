import re

def sanitize_filename(name: str, replacement: str = "_") -> str:
    """Removes characters illegal in filenames."""
    return re.sub(r'[\\/*?:"<>|]', replacement, name)

def slugify_album_filename(pattern: str, artist: str, album: str) -> str:
    """Formats the filename based on the config pattern."""
    filename = pattern.replace("{artist}", artist).replace("{album}", album)
    return sanitize_filename(filename)
