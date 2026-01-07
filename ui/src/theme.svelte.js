export const theme = $state({

  palette: {
    100: "#242424",
    200: "#323232",
    300: "#424242",
    400: "#CCCCCC",
    500: "#FFFFFF",
  },

  colors:{
    "background-main": "200",
    "background-drawer": "100",
    "text-main": "500",
    "text-muted": "400",
    "border-muted": "300",
  },

  layout: {
    "grid-gap-main": 18,
    "grid-cover-size": 200,
    "font-size-title": 14,
    "font-weight-title": 400,
    "font-size-artist": 12,
    "font-weight-artist": 400,
    "grid-text-gap-main": 8,
    "grid-text-gap-lesser": 4,
  },

});

export function getThemeVariables() {
  let styles = "";

  for (const [key, value] of Object.entries(theme.palette)) {
    styles += `--palette-${key}: ${value}; `;
  }

  for (const [key, paletteKey] of Object.entries(theme.colors)) {
    styles += `--${key}: var(--palette-${paletteKey}); `;
  }

for (const [key, value] of Object.entries(theme.layout)) {
    if (key.includes("weight")) {
      styles += `--${key}: ${value}; `;
    } else {
      styles += `--${key}: ${value}px; `;
    }
  }

  return styles;
}
