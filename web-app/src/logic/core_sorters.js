export const coreSorters = {
  default: {
    label: "Default",
    orderBy: "REGEXP_REPLACE(LOWER(COALESCE(data->>'$.tags.CUSTOM_ALBUMARTIST', data->>'$.ALBUMARTIST', '')), '^the ', '') ASC, data->>'$.DATE' ASC, LOWER(data->>'$.ALBUM') ASC"
  },
  date_added: {
    label: "Date Added",
    orderBy: "GREATEST(COALESCE(CAST(data->>'$.tags.UNIX_ADDED_YOUTUBE' AS BIGINT), 0), COALESCE(CAST(data->>'$.tags.UNIX_ADDED_APPLEMUSIC' AS BIGINT), 0), COALESCE(CAST(data->>'$.tags.UNIX_ADDED_FOOBAR' AS BIGINT), 0), COALESCE(CAST(data->>'$.unix_added' AS BIGINT), 0)) DESC"
  },
  az: {
    label: "Alphabetical",
    orderBy: "LOWER(data->>'$.ALBUM') ASC"
  },
  artist: {
    label: "Artist",
    orderBy: "REGEXP_REPLACE(LOWER(COALESCE(data->>'$.tags.CUSTOM_ALBUMARTIST', data->>'$.ALBUMARTIST', '')), '^the ', '') ASC"
  },
  year: {
    label: "Year",
    orderBy: "data->>'$.DATE' DESC"
  },
  duration: {
    label: "Duration",
    orderBy: "CAST(COALESCE(data->>'$.album_duration', '0') AS BIGINT) DESC"
  },
  chroma: {
    label: "Chroma",
    orderBy: "CAST(COALESCE(data->>'$.tags.COVER_CHROMA', '0') AS FLOAT) DESC"
  },
  entropy: {
    label: "Entropy",
    orderBy: "CAST(COALESCE(data->>'$.tags.COVER_ENTROPY', '0') AS FLOAT) DESC"
  }
};
