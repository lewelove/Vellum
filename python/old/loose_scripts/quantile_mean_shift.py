import sys
import cv2
import numpy as np
import matplotlib.pyplot as plt
from sklearn.cluster import MeanShift, estimate_bandwidth
from scipy.spatial import KDTree

def merge_centroids(centers, labels, threshold=10.0):
    """
    Aggressively merges cluster centers that are closer than the perceptual threshold.
    """
    if len(centers) <= 1:
        return centers, labels

    new_centers = []
    # Map old cluster index to new cluster index
    lookup = {}
    
    used = np.zeros(len(centers), dtype=bool)
    
    for i in range(len(centers)):
        if used[i]:
            continue
            
        # Current center is a new master cluster
        master_idx = len(new_centers)
        current_center = centers[i]
        new_centers.append(current_center)
        lookup[i] = master_idx
        used[i] = True
        
        # Check all other centers against this master
        for j in range(i + 1, len(centers)):
            if used[j]:
                continue
            
            dist = np.linalg.norm(current_center - centers[j])
            if dist < threshold:
                lookup[j] = master_idx
                used[j] = True
                # Optional: Move master center slightly toward the merged center
                # new_centers[master_idx] = (new_centers[master_idx] + centers[j]) / 2

    # Map the labels to the new compressed indices
    new_labels = np.array([lookup[l] for l in labels])
    return np.array(new_centers), new_labels

def main():
    if len(sys.argv) < 2:
        print("Usage: python actual_mean_shift.py <image_path>")
        sys.exit(1)

    path = sys.argv[1]
    img = cv2.imread(path)
    if img is None:
        print(f"Error: Could not read {path}")
        sys.exit(1)

    # 1. Dual Resize (Nearest Neighbor)
    discovery_img = cv2.resize(img, (64, 64), interpolation=cv2.INTER_NEAREST)
    discovery_lab = cv2.cvtColor(discovery_img, cv2.COLOR_BGR2LAB)
    discovery_pixels = discovery_lab.reshape((-1, 3)).astype(np.float32)

    ratio_img = cv2.resize(img, (256, 256), interpolation=cv2.INTER_NEAREST)
    ratio_lab = cv2.cvtColor(ratio_img, cv2.COLOR_BGR2LAB)
    ratio_pixels = ratio_lab.reshape((-1, 3)).astype(np.float32)

    # 2. Estimate Bandwidth
    # Quantile 0.15 is standard. We clamp to a tight range to prevent 'mud'.
    bw = estimate_bandwidth(discovery_pixels, quantile=0.15, n_samples=500)
    bw = np.clip(bw, 4.0, 16.0) 
    print(f"Bandwidth: {bw:.2f}")

    # 3. Mean Shift Clustering
    ms = MeanShift(bandwidth=bw, bin_seeding=True)
    ms.fit(discovery_pixels)
    
    centers = ms.cluster_centers_
    # Labels for the 64x64 discovery grid
    discovery_labels = ms.labels_

    print(f"Pre-merge clusters: {len(centers)}")

    # 4. AGGRESSIVE MERGE (Perceptual floor)
    # This is the 'fuck noise' step. 10.0 in Lab is the roughly 'distinct color' limit.
    final_centers, _ = merge_centroids(centers, discovery_labels, threshold=12.0)
    print(f"Post-merge clusters: {len(final_centers)}")

    # 5. ASSIGNMENT (Nearest Center for 256x256 grid)
    tree = KDTree(final_centers)
    _, ratio_labels = tree.query(ratio_pixels)

    # 6. Reconstruct Segmented Image
    segmented_pixels = final_centers[ratio_labels].reshape(ratio_lab.shape).astype(np.uint8)
    segmented_bgr = cv2.cvtColor(segmented_pixels, cv2.COLOR_LAB2BGR)
    segmented_rgb = cv2.cvtColor(segmented_bgr, cv2.COLOR_BGR2RGB)

    # 7. Calculate Ratios
    total_pixels_count = ratio_pixels.shape[0]
    palette = []
    for i in range(len(final_centers)):
        bgr = cv2.cvtColor(final_centers[i].reshape(1,1,3).astype(np.uint8), cv2.COLOR_LAB2BGR)[0][0]
        count = np.sum(ratio_labels == i)
        ratio = count / total_pixels_count
        palette.append({'bgr': bgr, 'ratio': ratio})
    
    palette.sort(key=lambda x: x['ratio'], reverse=True)

    # Terminal Output
    print("\nFinal Palette (Sorted by Representation):")
    for i, data in enumerate(palette):
        bgr = data['bgr']
        hex_color = "#{:02X}{:02X}{:02X}".format(bgr[2], bgr[1], bgr[0])
        print(f"  {i+1}: {hex_color} | Ratio: {data['ratio']:.4f}")
    print("")

    # Create Ratio Square (256x256)
    # Filling a 256x256 buffer pixel by pixel based on sorted ratios
    ratio_square_flat = np.zeros((256 * 256, 3), dtype=np.uint8)
    current_idx = 0
    for data in palette:
        num_pixels = int(round(data['ratio'] * (256 * 256)))
        end_idx = min(current_idx + num_pixels, 256 * 256)
        ratio_square_flat[current_idx:end_idx] = data['bgr']
        current_idx = end_idx
    
    # In case rounding left a few pixels empty at the end, fill with the last color
    if current_idx < 256 * 256:
        ratio_square_flat[current_idx:] = palette[-1]['bgr']
        
    ratio_square = ratio_square_flat.reshape((256, 256, 3))
    ratio_square_rgb = cv2.cvtColor(ratio_square, cv2.COLOR_BGR2RGB)

    # 8. Plot
    fig, (ax1, ax2, ax3) = plt.subplots(1, 3, figsize=(15, 5))
    ax1.imshow(cv2.cvtColor(ratio_img, cv2.COLOR_BGR2RGB))
    ax1.set_title("Original (256x256 Nearest)")
    ax1.axis('off')
    
    ax2.imshow(segmented_rgb)
    ax2.set_title(f"Segmented ({len(final_centers)} Clusters)")
    ax2.axis('off')

    ax3.imshow(ratio_square_rgb)
    ax3.set_title("Ratios (Sorted)")
    ax3.axis('off')
    
    plt.tight_layout()
    plt.show()

if __name__ == "__main__":
    main()
