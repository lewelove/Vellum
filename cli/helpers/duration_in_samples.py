def resolve(audio_obj) -> int:
    return getattr(audio_obj.info, 'total_samples', 0)
