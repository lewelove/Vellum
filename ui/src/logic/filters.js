// Pure JS Filtering Logic for ALBUMS
// The input 'item' is now an ALBUM object containing a 'tracks' array.

export const filters = {
  genre: (album, val) => album.GENRE === val,
  
  search: (album, val) => {
    const q = val.toLowerCase();
    
    // Check Album Metadata
    if (
      (album.ALBUM && album.ALBUM.toLowerCase().includes(q)) ||
      (album.ALBUMARTIST && album.ALBUMARTIST.toLowerCase().includes(q))
    ) {
      return true;
    }
    
    // Check Track Titles
    if (album.tracks && album.tracks.some(t => t.TITLE && t.TITLE.toLowerCase().includes(q))) {
      return true;
    }
    
    return false;
  },
  
  decade: (album, val) => {
    // val is "1990s"
    if (!album.DATE) return false;
    const year = parseInt(album.DATE.substring(0, 4));
    const start = parseInt(val.substring(0, 4));
    const end = start + 9;
    return year >= start && year <= end;
  },
  
  recent: (album) => {
    // Added in last 30 days
    const now = Math.floor(Date.now() / 1000);
    const added = parseInt(album.unix_added || 0);
    return (now - added) < (86400 * 30);
  }
};

export function applyFilter(album, filterKey, filterVal) {
  if (!filterKey || !filters[filterKey]) return true;
  return filters[filterKey](album, filterVal);
}
