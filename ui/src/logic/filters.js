export const filters = {
  genre: (album, val) => {
    if (Array.isArray(album.GENRE)) {
      return album.GENRE.includes(val);
    }
    return album.GENRE === val;
  },

  totaltracks: (album, val) => album.total_tracks === String(val),

  search: (album, val) => {
    const q = val.toLowerCase();

    if (
      (album.ALBUM && album.ALBUM.toLowerCase().includes(q)) ||
      (album.ALBUMARTIST && album.ALBUMARTIST.toLowerCase().includes(q))
    ) {
      return true;
    }

    if (album.tracks && album.tracks.some(t => t.TITLE && t.TITLE.toLowerCase().includes(q))) {
      return true;
    }

    return false;
  },

  decade: (album, val) => {
    if (!album.DATE) return false;
    const year = parseInt(album.DATE.substring(0, 4));
    const start = parseInt(val.substring(0, 4));
    const end = start + 9;
    return year >= start && year <= end;
  },

  recent: (album) => {
    const now = Math.floor(Date.now() / 1000);
    const added = parseInt(album.unix_added || 0);
    return (now - added) < (86400 * 30);
  },

  year_added: (album, val) => {
    const unix = parseInt(album.unix_added || 0);
    if (unix <= 0) return false;
    const year = new Date(unix * 1000).getFullYear().toString();
    return year === val;
  },

  month_added: (album, val) => {
    const unix = parseInt(album.unix_added || 0);
    if (unix <= 0) return false;
    const date = new Date(unix * 1000);
    const mVal = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
    return mVal === val;
  },

  chroma: (album, val) => {
    const score = parseFloat(album.cover_chroma || 0);
    switch (val) {
      case "Monochrome": return score === 0;
      case "Bleak":      return score > 0 && score < 15;
      case "Muted":      return score >= 15 && score < 33;
      case "Standard":   return score >= 33 && score < 60;
      case "Vibrant":    return score >= 60;
      default:           return true;
    }
  }
};

export function applyFilter(album, filterKey, filterVal) {
  if (!filterKey || !filters[filterKey]) return true;
  return filters[filterKey](album, filterVal);
}
