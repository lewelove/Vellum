// Pure JS Grouping Logic (Sidebar Generators)

export const groupers = {
  genre: (tracks) => {
    const map = new Map();
    tracks.forEach(t => {
      const g = t.GENRE || "Unknown";
      map.set(g, (map.get(g) || 0) + 1);
    });
    return map;
  },
  
  decade: (tracks) => {
    const map = new Map();
    tracks.forEach(t => {
      if (t.DATE && t.DATE.length >= 4) {
        const d = t.DATE.substring(0, 3) + "0s";
        map.set(d, (map.get(d) || 0) + 1);
      }
    });
    return map;
  }
};

export function generateSidebarGroup(tracks, groupKey) {
  if (!groupers[groupKey]) return [];
  
  const map = groupers[groupKey](tracks);
  const result = [];
  
  for (const [val, count] of map.entries()) {
    result.push({
      label: val,
      value: val,
      count: count,
      filterTarget: groupKey // Usually maps 1:1, can be customized
    });
  }
  
  // Sort alphabetically by default
  result.sort((a, b) => a.value.localeCompare(b.value));
  return result;
}
