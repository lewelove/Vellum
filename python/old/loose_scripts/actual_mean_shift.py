import sys
import cv2
import numpy as np
import matplotlib.pyplot as plt
from sklearn.cluster import MeanShift, estimate_bandwidth
from scipy.spatial import KDTree

if len(sys.argv) < 2:
    print("Usage: python actual_mean_shift.py <image_path> [manual_bandwidth]")
    sys.exit(1)

path = sys.argv[1]
manual_bw = float(sys.argv[2]) if len(sys.argv) > 2 else None

img = cv2.imread(path)
if img is None:
    print(f"Error: Could not read {path}")
    sys.exit(1)

h, w = img.shape[:2]
scale = 64 / max(h, w)
img_small = cv2.resize(img, (int(w * scale), int(h * scale)), interpolation=cv2.INTER_NEAREST)

img_lab = cv2.cvtColor(img_small, cv2.COLOR_BGR2LAB)
pixels = img_lab.reshape((-1, 3)).astype(np.float32)

if manual_bw:
    bandwidth = manual_bw
else:
    bandwidth = estimate_bandwidth(pixels, quantile=0.2, n_samples=500)
    if bandwidth == 0: bandwidth = 1.0

ms = MeanShift(bandwidth=bandwidth, bin_seeding=True)
ms.fit(pixels)

cluster_centers = ms.cluster_centers_
tree = KDTree(cluster_centers)
_, labels = tree.query(pixels)

total_pixels = pixels.shape[0]
centers_uint8 = np.uint8(cluster_centers)
centers_bgr = cv2.cvtColor(centers_uint8.reshape(-1, 1, 3), cv2.COLOR_LAB2BGR).reshape(-1, 3)

raw_palette = []
for i in range(len(cluster_centers)):
    bgr = centers_bgr[i]
    count = np.sum(labels == i)
    percentage = (count / total_pixels) * 100
    
    hsv = cv2.cvtColor(np.uint8([[bgr]]), cv2.COLOR_BGR2HSV)[0][0]
    vibrancy = int(hsv[1]) * int(hsv[2])
    
    raw_palette.append({
        'bgr': bgr, 
        'percentage': percentage,
        'vibrancy': vibrancy
    })

# Filter out colors < 0.1% and sort by representation
palette_data = [c for c in raw_palette if c['percentage'] >= 0.1]
palette_data.sort(key=lambda x: x['percentage'], reverse=True)

n_clusters_ = len(palette_data)

print(f"\nMean Shift found {len(raw_palette)} clusters total.")
print(f"Filtered to {n_clusters_} clusters (cutoff < 0.1% representation).")
print(f"Radius (Bandwidth): {bandwidth:.2f}\n")

for i, data in enumerate(palette_data):
    color = data['bgr']
    hex_c = "#{:02x}{:02x}{:02x}".format(color[2], color[1], color[0])
    print(f"{i+1}: HEX {hex_c} | Representation: {data['percentage']:.2f}% | Vibrancy: {data['vibrancy']}")

bar_h, bar_w = 120, 1000
palette_bar = np.zeros((bar_h, bar_w, 3), dtype=np.uint8)
if n_clusters_ > 0:
    step = bar_w // n_clusters_
    for i, data in enumerate(palette_data):
        start_x = i * step
        end_x = (i + 1) * step if i < n_clusters_ - 1 else bar_w
        palette_bar[:, start_x:end_x] = data['bgr']

img_rgb = cv2.cvtColor(img_small, cv2.COLOR_BGR2RGB)
bar_rgb = cv2.cvtColor(palette_bar, cv2.COLOR_BGR2RGB)

fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8), gridspec_kw={'height_ratios': [3, 1]})
ax1.imshow(img_rgb)
ax1.set_title(f"Original (64x64 Nearest Neighbor)")
ax1.axis('off')

ax2.imshow(bar_rgb)
ax2.set_title(f"Sorted by Representation (Filtered set)")
ax2.axis('off')

plt.tight_layout()
plt.show()
