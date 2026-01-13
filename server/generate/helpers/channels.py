def resolve(audio_obj) -> int:
    return getattr(audio_obj.info, 'channels', 0)
