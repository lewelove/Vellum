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
    "text-gap-lesser": 8,
    "drawer-gap-main": 0,
    "drawer-chevron-height": 12,
    "drawer-chevron-width": 24,
  },

  drawer: {
    "drawer-padding-y": 18,
    "drawer-padding-x": 24,
    "drawer-font-size-album": 18,
    "drawer-font-size-artist": 16,
    "drawer-font-size-track": 14,
    "drawer-track-y": 22,
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
      if (key.includes("weight")) {
        styles += `--${key}: ${value}; `;
      } else {
        styles += `--${key}: ${value}px; `;
      }
    }
  }

  return styles;
}
