def group_genre(ctx):
    # Returns raw SQL logic for grouping
    # Target filter matches 'filter_genre' defined in filtering/std.py
    return {
        "sql": 'SELECT GENRE as value, COUNT(*) as count FROM albums WHERE GENRE != "" GROUP BY GENRE ORDER BY GENRE ASC',
        "filter_target": "genre",
        "display_name": "By Genre"
    }

def group_albumartist(ctx):
    return {
        "sql": 'SELECT ALBUMARTIST as value, COUNT(*) as count FROM albums WHERE ALBUMARTIST != "" GROUP BY ALBUMARTIST ORDER BY ALBUMARTIST ASC',
        "filter_target": "search", # Using search filter for artist allows partial matching
        "display_name": "By Artist"
    }

def group_decade(ctx):
    # Python-based logic injection into SQL
    return {
        "sql": """
            SELECT 
                (SUBSTR(DATE, 1, 3) || '0s') as value, 
                COUNT(*) as count 
            FROM albums 
            WHERE DATE IS NOT NULL AND DATE != ""
            GROUP BY value 
            ORDER BY value DESC
        """,
        "filter_target": "decade",
        "display_name": "By Decade"
    }
