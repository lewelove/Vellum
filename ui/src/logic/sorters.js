const getSortableArtist = (name) => {
  if (!name) return "";
  const n = name.trim();
  if (n.toLowerCase().startsWith("the ")) {
    return n.slice(4).trim();
  }
  return n;
};

/**
 * Resolves the added date based on strict UI priority:
 * 1. UNIX_ADDED_YOUTUBE
 * 2. UNIX_ADDED_APPLEMUSIC
 * 3. UNIX_ADDED_FOOBAR
 * 4. Fallback to computed unix_added
 */
const getPriorityAddedDate = (album) => {
  const yt = parseInt(album.UNIX_ADDED_YOUTUBE) || 0;
  if (yt > 0) return yt;

  const am = parseInt(album.UNIX_ADDED_APPLEMUSIC) || 0;
  if (am > 0) return am;

  const fb = parseInt(album.UNIX_ADDED_FOOBAR) || 0;
  if (fb > 0) return fb;

  return parseInt(album.unix_added) || 0;
};

export const sorters = {
  default: (a, b) => {
    const artistA = getSortableArtist(a.CUSTOM_ALBUMARTIST).toLowerCase();
    const artistB = getSortableArtist(b.CUSTOM_ALBUMARTIST).toLowerCase();
    const artistComp = artistA.localeCompare(artistB);
    if (artistComp !== 0) return artistComp;

    const dateA = a.DATE || "0000";
    const dateB = b.DATE || "0000";
    const dateComp = dateA.localeCompare(dateB);
    if (dateComp !== 0) return dateComp;

    const titleA = (a.ALBUM || "").toLowerCase();
    const titleB = (b.ALBUM || "").toLowerCase();
    return titleA.localeCompare(titleB);
  },

  date_added: (a, b) => getPriorityAddedDate(b) - getPriorityAddedDate(a),
  
  az: (a, b) => (a.ALBUM || "").localeCompare(b.ALBUM || ""),
  
  artist: (a, b) => (a.CUSTOM_ALBUMARTIST || "").localeCompare(b.CUSTOM_ALBUMARTIST || ""),
  
  year: (a, b) => {
    const dateA = a.DATE || "0000";
    const dateB = b.DATE || "0000";
    return dateB.localeCompare(dateA);
  },

  entropy: (a, b) => (b.cover_entropy || 0) - (a.cover_entropy || 0),

  chroma: (a, b) => (b.cover_chroma || 0) - (a.cover_chroma || 0),

  duration: (a, b) => (b.album_duration_in_ms || 0) - (a.album_duration_in_ms || 0)
};

export const SORTER_LABELS = {
  default: "Default",
  az: "Alphabetical",
  year: "Year",
  date_added: "Date Added",
  duration: "Duration",
  chroma: "Chroma",
  entropy: "Entropy",
};
