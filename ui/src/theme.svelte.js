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

  "album-grid": {
    "gap-x": 20,
    "gap-y": 12,
    "cover-size": 200,
    "text-gap-main": 8,
    "text-gap-lesser": 4,
  }

});

export function getThemeVariables() {
  let styles = "";

  // Palette
  for (const [key, value] of Object.entries(theme.palette)) {
    styles += `--palette-${key}: ${value}; `;
  }

  // Color Mapping
  for (const [key, paletteKey] of Object.entries(theme.colors)) {
    styles += `--${key}: var(--palette-${paletteKey}); `;
  }

  // Typography & Grid Logic
  const numericCategories = ["typography", "album-grid"];
  
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
