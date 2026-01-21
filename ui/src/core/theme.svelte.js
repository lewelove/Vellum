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
    "gap-x": 18,
    "gap-y": 12,
    "cover-size": 200,
    "text-gap-main": 8,
    "text-gap-lesser": 2,
    "font-line-height-title": 18,
    "font-line-height-artist": 16,
    "drawer-gap-main": 0,
    "drawer-chevron-height": 12,
    "drawer-chevron-width": 24,
    // Material Design elevation (dp4 equivalent)
    "album-cover-shadow": "0px 0px 8px -1px rgba(0,0,0,0.2), 0px 0px 6px 0px rgba(0,0,0,0.2)"
    // "album-cover-shadow": "0px 0px 5px -1px rgba(0,0,0,0.4), 0px 0px 5px 0px rgba(0,0,0,0.2), 0px 0px 10px 0px rgba(0,0,0,0.15)"
  },

  drawer: {
    "drawer-padding-y": 18,
    "drawer-padding-x": 24,
    "drawer-font-size-album": 18,
    "drawer-font-size-artist": 16,
    "drawer-font-size-track": 14,
    "drawer-track-y": 22,
    "drawer-cover-size": 480,
    "drawer-split-gap": 24
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
      if (key.includes("weight") || key.includes("shadow")) { // Added shadow check
        styles += `--${key}: ${value}; `;
      } else {
        styles += `--${key}: ${value}px; `;
      }
    }
  }

  return styles;
}
