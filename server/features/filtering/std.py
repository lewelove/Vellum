def filter_genre(ctx):
    val = ctx.user_value
    return {
        "where": "GENRE = ?",
        "params": [val]
    }

def filter_search(ctx):
    val = ctx.user_value
    # Simple search across Title, Artist, Album
    clean_val = f"%{val}%"
    return {
        "where": "(TITLE LIKE ? OR ALBUMARTIST LIKE ? OR ALBUM LIKE ?)",
        "params": [clean_val, clean_val, clean_val]
    }

def filter_decade(ctx):
    # Expects input like "1990s"
    val = str(ctx.user_value)
    if not val or len(val) < 4:
        return {"where": "1=1", "params": []}
        
    start_year = val[:3] + "0"
    end_year = val[:3] + "9"
    
    # Using string comparison for dates YYYY-MM
    return {
        "where": "(DATE >= ? AND DATE <= ?)",
        "params": [start_year, f"{end_year}-12"]
    }

def filter_date_added_recent(ctx):
    # Example of a parameter-less filter (Last 30 days)
    # Logic: UNIX timestamp comparison
    import time
    cutoff = int(time.time()) - (86400 * 30)
    return {
        "where": "CAST(unix_added AS INTEGER) > ?",
        "params": [cutoff]
    }
