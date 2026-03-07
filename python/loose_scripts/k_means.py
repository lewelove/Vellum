import sys
import cv2
import numpy as np
import matplotlib.pyplot as plt

if len(sys.argv) < 2:
    print("Usage: python k_means.py <image_path> [k_colors]")
    sys.exit(1)

path = sys.argv[1]
k = int(sys.argv[2]) if len(sys.argv) > 2 else 8

img = cv2.imread(path)
if img is None:
    print(f"Error: Could not open {path}")
    sys.exit(1)

# Resize
h, w = img.shape[:2]
scale = 400 / max(h, w)
img_small = cv2.resize(img, (int(w * scale), int(h * scale)))

# K-Means Math
pixels = img_small.reshape((-1, 3)).astype(np.float32)
criteria = (cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER, 100, 0.2)
_, labels, centers = cv2.kmeans(pixels, k, None, criteria, 10, cv2.KMEANS_RANDOM_CENTERS)

# Extract Palette
centers = np.uint8(centers)
_, counts = np.unique(labels, return_counts=True)
indices = np.argsort(-counts)
palette = centers[indices]

print(f"K-Means Palette (k={k}):")
for i, color in enumerate(palette):
    hex_c = "#{:02x}{:02x}{:02x}".format(color[2], color[1], color[0])
    print(f"{i+1}: BGR{tuple(color)} | {hex_c}")

# Build visual bar
bar = np.zeros((100, 600, 3), dtype=np.uint8)
step = 600 // k
for i in range(k):
    bar[:, i*step:(i+1)*step] = palette[i]

# --- MATPLOTLIB GUI REPLACEMENT ---
# Convert BGR (OpenCV) to RGB (Matplotlib)
img_rgb = cv2.cvtColor(img_small, cv2.COLOR_BGR2RGB)
bar_rgb = cv2.cvtColor(bar, cv2.COLOR_BGR2RGB)

# Create a clean window with two rows
fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(6, 6))

ax1.imshow(img_rgb)
ax1.set_title("Original")
ax1.axis('off')

ax2.imshow(bar_rgb)
ax2.set_title("Extracted Palette")
ax2.axis('off')

plt.tight_layout()
plt.show() # This opens the window natively on NixOS
