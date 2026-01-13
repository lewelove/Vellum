def resolve(audio_obj) -> int:
    return getattr(audio_obj.info, 'bits_per_sample', 0)
