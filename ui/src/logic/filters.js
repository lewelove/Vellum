// Pure JS Filtering Logic

export const filters = {
  genre: (track, val) => track.GENRE === val,
  
  search: (track, val) => {
    const q = val.toLowerCase();
    return (
      (track.TITLE && track.TITLE.toLowerCase().includes(q)) ||
      (track.ALBUM && track.ALBUM.toLowerCase().includes(q)) ||
      (track.ALBUMARTIST && track.ALBUMARTIST.toLowerCase().includes(q))
    );
  },
  
  decade: (track, val) => {
    // val is "1990s"
    if (!track.DATE) return false;
    const year = parseInt(track.DATE.substring(0, 4));
    const start = parseInt(val.substring(0, 4));
    const end = start + 9;
    return year >= start && year <= end;
  },
  
  recent: (track) => {
    // Added in last 30 days
    const now = Math.floor(Date.now() / 1000);
    const added = parseInt(track.unix_added || 0);
    return (now - added) < (86400 * 30);
  }
};

export function applyFilter(track, filterKey, filterVal) {
  if (!filterKey || !filters[filterKey]) return true;
  return filters[filterKey](track, filterVal);
}
