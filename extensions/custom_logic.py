def resolve_helper_is_cd(ctx):
    # Access the audio physics via 'audio_obj' (Mutagen object)
    # Note: 'audio_obj' might be None if the file is missing/corrupt
    audio = ctx.get("audio_obj")
    
    if audio and hasattr(audio, "info"):
        if audio.info.sample_rate == 44100 or audio.info.bits_per_sample == 16:
            return True
            
    return False
