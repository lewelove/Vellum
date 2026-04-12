export const coreFacets = {
  genre: {
    label: "Genre",
    select: "unnest(CAST(COALESCE(data->'$.GENRE', '[\"Unknown\"]') AS VARCHAR[]))",
    filterWhere: (val) => val === 'Unknown' ? `data->'$.GENRE' IS NULL` : `list_contains(CAST(data->'$.GENRE' AS VARCHAR[]), '${val.replace(/'/g, "''")}')`,
    orderBy: "count DESC"
  },
  decade: {
    label: "Decade",
    select: "SUBSTR(data->>'$.DATE', 1, 3) || '0s'",
    filterWhere: (val) => `SUBSTR(data->>'$.DATE', 1, 3) || '0s' = '${val.replace(/'/g, "''")}'`,
    orderBy: "value DESC"
  },
  year_added: {
    label: "Year Added",
    select: "strftime(to_timestamp(CAST(COALESCE(data->>'$.unix_added', '0') AS BIGINT)), '%Y')",
    filterWhere: (val) => `strftime(to_timestamp(CAST(COALESCE(data->>'$.unix_added', '0') AS BIGINT)), '%Y') = '${val.replace(/'/g, "''")}'`,
    orderBy: "value DESC"
  },
  month_added: {
    label: "Month Added",
    select: "strftime(to_timestamp(CAST(COALESCE(data->>'$.unix_added', '0') AS BIGINT)), '%Y-%m')",
    filterWhere: (val) => `strftime(to_timestamp(CAST(COALESCE(data->>'$.unix_added', '0') AS BIGINT)), '%Y-%m') = '${val.replace(/'/g, "''")}'`,
    orderBy: "value DESC",
    getLabel: (val) => {
      if (!val) return "Unknown";
      const[y, m] = val.split('-');
      const date = new Date(y, parseInt(m) - 1);
      const monthNames =["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
      return `${monthNames[date.getMonth()]} ${y}`;
    }
  },
  totaltracks: {
    label: "Total Tracks",
    select: "COALESCE(CAST(data->>'$.total_tracks' AS VARCHAR), '0')",
    filterWhere: (val) => `COALESCE(CAST(data->>'$.total_tracks' AS VARCHAR), '0') = '${val.replace(/'/g, "''")}'`,
    orderBy: "CAST(value AS INTEGER) ASC"
  },
  chroma: {
    label: "Chroma",
    select: `CASE 
      WHEN CAST(COALESCE(data->>'$.tags.COVER_CHROMA', '0') AS FLOAT) = 0 THEN 'Monochrome'
      WHEN CAST(COALESCE(data->>'$.tags.COVER_CHROMA', '0') AS FLOAT) < 15 THEN 'Bleak'
      WHEN CAST(COALESCE(data->>'$.tags.COVER_CHROMA', '0') AS FLOAT) < 33 THEN 'Muted'
      WHEN CAST(COALESCE(data->>'$.tags.COVER_CHROMA', '0') AS FLOAT) < 60 THEN 'Standard'
      ELSE 'Vibrant'
    END`,
    filterWhere: (val) => `(CASE 
      WHEN CAST(COALESCE(data->>'$.tags.COVER_CHROMA', '0') AS FLOAT) = 0 THEN 'Monochrome'
      WHEN CAST(COALESCE(data->>'$.tags.COVER_CHROMA', '0') AS FLOAT) < 15 THEN 'Bleak'
      WHEN CAST(COALESCE(data->>'$.tags.COVER_CHROMA', '0') AS FLOAT) < 33 THEN 'Muted'
      WHEN CAST(COALESCE(data->>'$.tags.COVER_CHROMA', '0') AS FLOAT) < 60 THEN 'Standard'
      ELSE 'Vibrant'
    END) = '${val.replace(/'/g, "''")}'`,
    orderBy: `CASE value 
      WHEN 'Vibrant' THEN 1 
      WHEN 'Standard' THEN 2 
      WHEN 'Muted' THEN 3 
      WHEN 'Bleak' THEN 4 
      WHEN 'Monochrome' THEN 5 
    END ASC`
  }
};
