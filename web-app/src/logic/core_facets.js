export const coreFacets = {
  genre: {
    label: "Genre",
    getValue: (a) => a.GENRE || "Unknown"
  },
  decade: {
    label: "Decade",
    getValue: (a) => {
      if (a.DATE && a.DATE.length >= 4) {
        return a.DATE.substring(0, 3) + "0s";
      }
      return null;
    }
  },
  year_added: {
    label: "Year Added",
    getValue: (a) => {
      const unix = parseInt(a.unix_added || 0);
      if (unix > 0) {
        return new Date(unix * 1000).getFullYear().toString();
      }
      return null;
    },
    sortBuckets: (map) => {
      return Array.from(map.entries()).sort((a, b) => b[0].localeCompare(a[0]));
    }
  },
  month_added: {
    label: "Month Added",
    getValue: (a) => {
      const unix = parseInt(a.unix_added || 0);
      if (unix > 0) {
        const date = new Date(unix * 1000);
        return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
      }
      return null;
    },
    getLabel: (val) => {
      if (!val) return "Unknown";
      const [y, m] = val.split('-');
      const date = new Date(y, parseInt(m) - 1);
      const monthNames = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
      return `${monthNames[date.getMonth()]} ${y}`;
    },
    sortBuckets: (map) => {
      return Array.from(map.entries()).sort((a, b) => b[0].localeCompare(a[0]));
    }
  },
  totaltracks: {
    label: "Total Tracks",
    getValue: (a) => String(a.total_tracks || "0")
  },
  chroma: {
    label: "Chroma",
    getValue: (a) => {
      const val = parseFloat(a.tags?.COVER_CHROMA || 0);
      if (val === 0) return "Monochrome";
      if (val < 15) return "Bleak";
      if (val < 33) return "Muted";
      if (val < 60) return "Standard";
      return "Vibrant";
    },
    sortBuckets: (map) => {
      const order = ["Vibrant", "Standard", "Muted", "Bleak", "Monochrome"];
      return Array.from(map.entries()).sort((a, b) => order.indexOf(a[0]) - order.indexOf(b[0]));
    }
  }
};
