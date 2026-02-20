def resolve_track_helper_is_cd(ctx):
    # Technical properties provided in 'physics'
    physics = ctx.get("physics")
    
    if physics:
        sample_rate = physics.get("sample_rate", 0)
        bit_depth = physics.get("bit_depth", 0)
        
        if sample_rate == 44100 or bit_depth == 16:
            return True
            
    return False
