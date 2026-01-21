// Pure JS Grouping Logic for ALBUMS
// Iterates over Albums to create Sidebar counts.

export const groupers = {

  genre: (albums) => {
    const map = new Map();
    albums.forEach(a => {
      const g = a.GENRE || "Unknown";
      map.set(g, (map.get(g) || 0) + 1);
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

};

export const GROUPER_LABELS = {
  genre: "Genre",
  decade: "Decade",
  totaltracks: "Total Tracks",
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
  
  result.sort((a, b) => a.value.localeCompare(b.value, undefined, { numeric: true }));
  return result;
}
