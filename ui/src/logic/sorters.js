// Pure JS Sorting Logic for ALBUMS

export const sorters = {
  date_added: (a, b) => (b.unix_added || 0) - (a.unix_added || 0), // DESC
  
  az: (a, b) => (a.ALBUM || "").localeCompare(b.ALBUM || ""),
  
  artist: (a, b) => (a.ALBUMARTIST || "").localeCompare(b.ALBUMARTIST || ""),
  
  year: (a, b) => {
    const dateA = a.DATE || "0000";
    const dateB = b.DATE || "0000";
    return dateB.localeCompare(dateA); // DESC
  }
};
