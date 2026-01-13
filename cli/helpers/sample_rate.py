def resolve(audio_obj) -> int:
    return getattr(audio_obj.info, 'sample_rate', 0)
