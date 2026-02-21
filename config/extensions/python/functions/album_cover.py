# import os
# import numpy as np
# from pathlib import Path
# from PIL import Image
#
# def resolve_album_helper_cover_entropy(ctx):
#     # Retrieve the cover_hash calculated by Rust Pass 1
#     c_hash = ctx.get("album", {}).get("cover_hash")
#     if not c_hash:
#         return 0
#
#     try:
#         # Access global configuration passed from the bridge
#         config = ctx.get("config", {})
#         thumb_dir_raw = config.get("storage", {}).get("thumbnail_cache_folder", "")
#         if not thumb_dir_raw:
#             return 0
#
#         thumb_dir = Path(thumb_dir_raw).expanduser()
#         thumb_file = thumb_dir / f"{c_hash}.png"
#
#         if thumb_file.exists():
#             return os.path.getsize(thumb_file)
#     except Exception:
#         pass
#
#     return 0
#
# def resolve_album_helper_cover_chroma(ctx):
#     # Retrieve the cover_hash calculated by Rust Pass 1
#     c_hash = ctx.get("album", {}).get("cover_hash")
#     if not c_hash:
#         return 0.0
#
#     try:
#         config = ctx.get("config", {})
#         thumb_dir_raw = config.get("storage", {}).get("thumbnail_cache_folder", "")
#         if not thumb_dir_raw:
#             return 0.0
#
#         thumb_dir = Path(thumb_dir_raw).expanduser()
#         thumb_file = thumb_dir / f"{c_hash}.png"
#
#         if not thumb_file.exists():
#             return 0.0
#
#         with Image.open(thumb_file) as img:
#             img = img.convert("RGB")
#             arr = np.array(img).astype(float)
#
#             r, g, b = arr[:, :, 0], arr[:, :, 1], arr[:, :, 2]
#
#             rg = r - g
#             yb = 0.5 * (r + g) - b
#
#             std_rg = np.std(rg)
#             std_yb = np.std(yb)
#
#             mean_rg = np.mean(rg)
#             mean_yb = np.mean(yb)
#
#             std_root = np.sqrt(std_rg**2 + std_yb**2)
#             mean_root = np.sqrt(mean_rg**2 + mean_yb**2)
#
#             return float(std_root + (0.3 * mean_root))
#
#     except Exception:
#         pass
#
#     return 0.0
