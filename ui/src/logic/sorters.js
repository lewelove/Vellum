// Pure JS Sorting Logic

export const sorters = {
  date_added: (a, b) => (b.unix_added || 0) - (a.unix_added || 0), // DESC
  
  az: (a, b) => (a.title || "").localeCompare(b.title || ""),
  
  artist: (a, b) => (a.artist || "").localeCompare(b.artist || ""),
  
  year: (a, b) => {
    const dateA = a.tracks[0]?.DATE || "0000";
    const dateB = b.tracks[0]?.DATE || "0000";
    return dateB.localeCompare(dateA); // DESC
  }
};
