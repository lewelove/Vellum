def sort_date_added(ctx):
    direction = ctx.user_value or "DESC"
    # Secondary sort by Album ID to ensure stable pagination
    return f"date_added {direction}, id ASC"

def sort_az(ctx):
    direction = ctx.user_value or "ASC"
    return f"ALBUM {direction}, id ASC"

def sort_artist(ctx):
    direction = ctx.user_value or "ASC"
    return f"ALBUMARTIST {direction}, ALBUM ASC, id ASC"

def sort_year(ctx):
    direction = ctx.user_value or "DESC"
    return f"DATE {direction}, ALBUM ASC, id ASC"
