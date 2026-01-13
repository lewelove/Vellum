def resolve(audio_obj) -> str:
    cls_name = audio_obj.__class__.__name__
    if "FLAC" in cls_name: return "FLAC"
    return "UNKNOWN"
