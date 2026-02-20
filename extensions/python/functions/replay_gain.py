def resolve_album_tag_replaygain_album_gain(ctx):
    return str(ctx["source"].get("REPLAYGAIN_ALBUM_GAIN", ""))

def resolve_album_tag_replaygain_album_peak(ctx):
    return str(ctx["source"].get("REPLAYGAIN_ALBUM_PEAK", ""))

def resolve_track_tag_replaygain_track_gain(ctx):
    return str(ctx["source"].get("REPLAYGAIN_TRACK_GAIN", ""))

def resolve_track_tag_replaygain_track_peak(ctx):
    return str(ctx["source"].get("REPLAYGAIN_TRACK_PEAK", ""))
