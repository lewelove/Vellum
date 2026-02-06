export const groupers = {

  genre: (albums) => {
    const map = new Map();
    albums.forEach(a => {
      const val = a.GENRE;
      const genres = Array.isArray(val) ? val : [val || "Unknown"];
      
      genres.forEach(g => {
        map.set(g, (map.get(g) || 0) + 1);
      });
    });
    return map;
  },
  
  decade: (albums) => {
    const map = new Map();
    albums.forEach(a => {
      if (a.DATE && a.DATE.length >= 4) {
        const d = a.DATE.substring(0, 3) + "0s";
        map.set(d, (map.get(d) || 0) + 1);
      }
    });
    return map;
  },

  totaltracks: (albums) => {
    const map = new Map();
    albums.forEach(a => {
      const t = a.TOTALTRACKS || "0";
      map.set(t, (map.get(t) || 0) + 1);
    });
    return map;
  },

  chroma: (albums) => {
    const map = new Map();
    const buckets = [
      { label: "Monochrome", threshold: 4 },
      { label: "Muted", threshold: 12 },
      { label: "Standard", threshold: 30 },
      { label: "Vibrant", threshold: Infinity }
    ];

    albums.forEach(a => {
      const val = parseFloat(a.cover_chroma || 0);
      const bucket = buckets.find(b => val < b.threshold);
      if (bucket) {
        map.set(bucket.label, (map.get(bucket.label) || 0) + 1);
      }
    });

    return map;
  }
};

export const GROUPER_LABELS = {
  genre: "Genre",
  decade: "Decade",
  totaltracks: "Total Tracks",
  chroma: "Chromaticity"
};

export function generateSidebarGroup(albums, groupKey) {
  if (!groupers[groupKey]) return [];
  
  const map = groupers[groupKey](albums);
  const result = [];
  
  for (const [val, count] of map.entries()) {
    result.push({
      label: val,
      value: val,
      count: count,
      filterTarget: groupKey
    });
  }
  
  if (groupKey === "chroma") {
    const order = ["Monochrome", "Muted", "Standard", "Vibrant"];
    result.sort((a, b) => order.indexOf(a.label) - order.indexOf(b.label));
  } else {
    result.sort((a, b) => a.value.localeCompare(b.value, undefined, { numeric: true }));
  }

  return result;
}
