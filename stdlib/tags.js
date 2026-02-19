const _formatHumanDate = (yyyy_mm) => {
  if (!yyyy_mm || yyyy_mm === "0000-00") return "Unknown Date";
  const [year, month] = yyyy_mm.split("-");
  if (!month || month === "00") return year;
  
  const months = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
  ];
  const monthIdx = parseInt(month, 10) - 1;
  return months[monthIdx] ? `${months[monthIdx]} ${year}` : year;
};

export const albumTags = {
  ALBUMARTIST: (ctx) => String(ctx.source.ALBUMARTIST || "Unknown"),
  ALBUM: (ctx) => String(ctx.source.ALBUM || "Unknown"),
  
  GENRE: (ctx) => {
    const raw = ctx.source.GENRE || "Unknown";
    const parts = Array.isArray(raw) 
      ? raw.map(v => String(v).trim())
      : String(raw).split(";").map(v => v.trim());
    return [...new Set(parts.filter(Boolean))];
  },

  DATE: (ctx) => {
    const candidates = ["DATE", "YEAR", "ORIGINALYEAR"];
    for (const key of candidates) {
      if (ctx.source[key]) return String(ctx.source[key]);
    }
    return "0000";
  },

  ORIGINAL_YYYY_MM: (ctx) => {
    const candidates = ["ORIGINAL_YYYY_MM", "ORIGINALYEARMONTH"];
    for (const key of candidates) {
      if (ctx.source[key]) return String(ctx.source[key]);
    }
    const date = albumTags.DATE(ctx);
    return `${date.substring(0, 4)}-00`;
  },

  ORIGINAL_YEAR: (ctx) => albumTags.ORIGINAL_YYYY_MM(ctx).substring(0, 4),
  ORIGINAL_DATE: (ctx) => _formatHumanDate(albumTags.ORIGINAL_YYYY_MM(ctx)),

  RELEASE_YYYY_MM: (ctx) => {
    if (ctx.source.RELEASE_YYYY_MM) return String(ctx.source.RELEASE_YYYY_MM);
    const date = albumTags.DATE(ctx);
    return `${date.substring(0, 4)}-00`;
  },

  RELEASE_YEAR: (ctx) => albumTags.RELEASE_YYYY_MM(ctx).substring(0, 4),
  RELEASE_DATE: (ctx) => _formatHumanDate(albumTags.RELEASE_YYYY_MM(ctx)),
  
  COMMENT: (ctx) => String(ctx.source.COMMENT || "")
};

export const trackTags = {
  TITLE: (ctx) => String(ctx.source.TITLE || "Untitled"),
  ARTIST: (ctx) => String(ctx.source.ARTIST || ctx.source.ALBUMARTIST || "Unknown"),
  TRACKNUMBER: (ctx) => String(ctx.source.TRACKNUMBER || ""),
  DISCNUMBER: (ctx) => String(ctx.source.DISCNUMBER || "1"),
  LYRICS: (ctx) => String(ctx.source.LYRICS || "")
};
