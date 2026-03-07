import sys
import cv2
import numpy as np
import matplotlib.pyplot as plt

if len(sys.argv) < 2:
    print("Usage: python palette_extractor.py <image_path>")
    sys.exit(1)

path = sys.argv[1]
img = cv2.imread(path)
if img is None:
    print(f"Error: Could not read {path}")
    sys.exit(1)

# 1. Resize (100x100 is fast but keeps accent colors alive)
h, w = img.shape[:2]
scale = 100 / max(h, w)
img_small = cv2.resize(img, (int(w * scale), int(h * scale)), interpolation=cv2.INTER_AREA)

# 2. Convert to LAB Color Space (Crucial for human-eye accuracy)
img_lab = cv2.cvtColor(img_small, cv2.COLOR_BGR2LAB)
pixels = img_lab.reshape((-1, 3)).astype(np.float32)

# 3. K-Means Clustering
k = 8
criteria = (cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER, 100, 0.2)
# We ask for K distinct clusters
_, labels, centers = cv2.kmeans(pixels, k, None, criteria, 10, cv2.KMEANS_RANDOM_CENTERS)

# 4. Convert centers back to BGR so we can display them
centers_uint8 = np.uint8(centers)
centers_bgr = cv2.cvtColor(centers_uint8.reshape(k, 1, 3), cv2.COLOR_LAB2BGR).reshape(k, 3)

# Count how many pixels belong to each cluster
_, counts = np.unique(labels, return_counts=True)

# 5. Sort by Vibrancy (Accent colors first, blacks/grays last)
palette_data = []
for i in range(k):
    bgr = centers_bgr[i]
    # Convert single color to HSV to measure colorfulness
    hsv = cv2.cvtColor(np.uint8([[bgr]]), cv2.COLOR_BGR2HSV)[0][0]
    
    saturation = int(hsv[1])
    brightness = int(hsv[2])
    vibrancy = saturation * brightness # High saturation + High brightness = Vibrant
    
    palette_data.append({
        'bgr': bgr,
        'count': counts[i],
        'vibrancy': vibrancy
    })

# Sort descending by vibrancy
palette_data.sort(key=lambda x: x['vibrancy'], reverse=True)
palette = [x['bgr'] for x in palette_data]

# Print Results
print(f"\nExtracted Palette (Sorted by Vibrancy):")
for i, data in enumerate(palette_data):
    color = data['bgr']
    hex_c = "#{:02x}{:02x}{:02x}".format(color[2], color[1], color[0])
    print(f"{i+1}: HEX {hex_c} | BGR {tuple(color)} | Pixels: {data['count']}")

# Build visual bar
bar_h, bar_w = 100, 600
palette_bar = np.zeros((bar_h, bar_w, 3), dtype=np.uint8)
step = bar_w // k

for i in range(k):
    end_idx = (i+1)*step if i < k - 1 else bar_w
    palette_bar[:, i*step:end_idx] = palette[i]

# --- MATPLOTLIB GUI ---
img_rgb = cv2.cvtColor(img_small, cv2.COLOR_BGR2RGB)
bar_rgb = cv2.cvtColor(palette_bar, cv2.COLOR_BGR2RGB)

fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(6, 7))

ax1.imshow(img_rgb)
ax1.set_title("Original (Downsampled)")
ax1.axis('off')

ax2.imshow(bar_rgb)
ax2.set_title(f"Extracted Palette (K={k}, Sorted by Vibrancy)")
ax2.axis('off')

plt.tight_layout()
plt.show()
