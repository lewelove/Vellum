export const theme = $state({

  palette: {
    100: "#242424",
    200: "#323232",
    300: "#424242",
    400: "#CCCCCC",
    500: "#FFFFFF",
  },

  colors: {
    "background-main": "200",
    "background-drawer": "100",
    "text-main": "500",
    "text-muted": "400",
    "border-muted": "300",
  },

  typography: {
    "font-size-title": 14,
    "font-weight-title": 400,
    "font-size-artist": 12,
    "font-weight-artist": 400,
  },

  albumGrid: {
    "crease-height": 18,
    "gap-x": 28,
    "gap-y": 18,
    "cover-size": 200,
    "text-gap-main": 10,
    "text-gap-lesser": 4,
    "font-line-height-title": 16,
    "font-line-height-artist": 14,
    "drawer-gap-main": 0,
    "drawer-chevron-height": 12,
    "drawer-chevron-width": 24,
    "album-cover-shadow": "0px 0px 10px -1px rgba(0,0,0,0.3), 0px 0px 8px 0px rgba(0,0,0,0.4)",
    "panel-shadow": "0px 0px 8px 0px rgba(0,0,0,0.2), 0px 0px 6px 0px rgba(0,0,0,0.2)",
    "button-shadow": "0px 0px 4px 0px rgba(0,0,0,0.2), 0px 0px 2px 0px rgba(0,0,0,0.2)"
  },

  drawer: {
    "drawer-padding-y": 18,
    "drawer-padding-x": 18,
    "drawer-font-size-album": 21,
    "drawer-font-size-artist": 18,
    "drawer-font-size-track": 14,
    "drawer-track-y": 32,
    // "drawer-cover-size": 400, // This is now a fallback, logic overrides it
    "drawer-split-gap": 24,
    "drawer-contents-x-max": 1600
  }

});

export function getThemeVariables() {
  let styles = "";

  for (const [key, value] of Object.entries(theme.palette)) {
    styles += `--palette-${key}: ${value}; `;
  }

  for (const [key, paletteKey] of Object.entries(theme.colors)) {
    styles += `--${key}: var(--palette-${paletteKey}); `;
  }

  const numericCategories = ["typography", "albumGrid", "drawer"];
  
  for (const cat of numericCategories) {
    for (const [key, value] of Object.entries(theme[cat])) {
      if (key.includes("weight") || key.includes("shadow")) {
        styles += `--${key}: ${value}; `;
      } else {
        styles += `--${key}: ${value}px; `;
      }
    }
  }

  return styles;
}
