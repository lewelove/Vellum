const getSortableArtist = (name) => {
  if (!name) return "";
  const n = String(name).trim();
  if (n.toLowerCase().startsWith("the ")) {
    return n.slice(4).trim();
  }
  return n;
};

const getPriorityAddedDate = (album) => {
  const t = album.tags || {};
  const yt = parseInt(t.UNIX_ADDED_YOUTUBE) || 0;
  if (yt > 0) return yt;

  const am = parseInt(t.UNIX_ADDED_APPLEMUSIC) || 0;
  if (am > 0) return am;

  const fb = parseInt(t.UNIX_ADDED_FOOBAR) || 0;
  if (fb > 0) return fb;

  return parseInt(album.unix_added) || 0;
};

export const sorters = {
  default: (a, b) => {
    const artistA = getSortableArtist(a.tags?.CUSTOM_ALBUMARTIST || a.ALBUMARTIST).toLowerCase();
    const artistB = getSortableArtist(b.tags?.CUSTOM_ALBUMARTIST || b.ALBUMARTIST).toLowerCase();
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
  
  artist: (a, b) => {
    const artA = a.tags?.CUSTOM_ALBUMARTIST || a.ALBUMARTIST || "";
    const artB = b.tags?.CUSTOM_ALBUMARTIST || b.ALBUMARTIST || "";
    return artA.localeCompare(artB);
  },
  
  year: (a, b) => {
    const dateA = a.DATE || "0000";
    const dateB = b.DATE || "0000";
    return dateB.localeCompare(dateA);
  },

  entropy: (a, b) => (b.tags?.COVER_ENTROPY || 0) - (a.tags?.COVER_ENTROPY || 0),

  chroma: (a, b) => (b.tags?.COVER_CHROMA || 0) - (a.tags?.COVER_CHROMA || 0),

  duration: (a, b) => (b.album_duration || 0) - (a.album_duration || 0)
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
