def resolve(audio_obj) -> int:
    length = getattr(audio_obj.info, 'length', 0)
    return int(length * 1000)
