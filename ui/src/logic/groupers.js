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

  year_added: (albums) => {
    const map = new Map();
    albums.forEach(a => {
      const unix = parseInt(a.unix_added || 0);
      if (unix > 0) {
        const year = new Date(unix * 1000).getFullYear().toString();
        map.set(year, (map.get(year) || 0) + 1);
      }
    });
    return map;
  },

  month_added: (albums) => {
    const map = new Map();
    const monthNames = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    albums.forEach(a => {
      const unix = parseInt(a.unix_added || 0);
      if (unix > 0) {
        const date = new Date(unix * 1000);
        const year = date.getFullYear();
        const monthNum = date.getMonth() + 1;
        const key = `${year}-${String(monthNum).padStart(2, '0')}`;
        const label = `${monthNames[date.getMonth()]} ${year}`;

        if (!map.has(key)) {
          map.set(key, { label, count: 0 });
        }
        map.get(key).count++;
      }
    });
    return map;
  },

  totaltracks: (albums) => {
    const map = new Map();
    albums.forEach(a => {
      const t = a.total_tracks || "0";
      map.set(t, (map.get(t) || 0) + 1);
    });
    return map;
  },

  chroma: (albums) => {
    const map = new Map();
    const buckets = [
      { label: "Monochrome", threshold: 0.0001 },
      { label: "Bleak", threshold: 15 },
      { label: "Muted", threshold: 33 },
      { label: "Standard", threshold: 60 },
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
  year_added: "Year Added",
  month_added: "Month Added",
  totaltracks: "Total Tracks",
  chroma: "Chromaticity"
};

export function generateSidebarGroup(albums, groupKey) {
  if (!groupers[groupKey]) return [];

  const map = groupers[groupKey](albums);
  const result = [];

  for (const [val, data] of map.entries()) {
    if (groupKey === "month_added") {
      result.push({
        label: data.label,
        value: val,
        count: data.count,
        filterTarget: groupKey
      });
    } else {
      result.push({
        label: val,
        value: val,
        count: data,
        filterTarget: groupKey
      });
    }
  }

  if (groupKey === "chroma") {
    const order = ["Vibrant", "Standard", "Muted", "Bleak", "Monochrome"];
    result.sort((a, b) => order.indexOf(a.label) - order.indexOf(b.label));
  } else if (groupKey === "year_added" || groupKey === "month_added") {
    result.sort((a, b) => b.value.localeCompare(a.value));
  } else {
    result.sort((a, b) => a.value.localeCompare(b.value, undefined, { numeric: true }));
  }

  return result;
}
