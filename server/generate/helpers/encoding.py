def resolve(audio_obj) -> str:
    # Mutagen audio objects have suffixes or we can infer from class name
    cls_name = audio_obj.__class__.__name__
    if "FLAC" in cls_name: return "FLAC"
    return "UNKNOWN"
