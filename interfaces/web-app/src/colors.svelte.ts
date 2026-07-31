export interface PaletteConfig {
  ok100: string;
  ok200: string;
  ok300: string;
  ok400: string;
  ok500: string;
}

export const colorsState = $state<{
  palette: PaletteConfig;
}>({
  palette: {
    ok100: "oklch(1.00 0 0)",
    ok200: "oklch(0.32 0 0)",
    ok300: "oklch(0.26 0 0)",
    ok400: "oklch(0.20 0 0)",
    ok500: "oklch(0.14 0 0)"
  }
});

export function applyColors(configData: any) {
  if (!configData) return;

  if (configData.palette) {
    Object.assign(colorsState.palette, configData.palette);
  }

  if (typeof document === "undefined") return;
  const root = document.documentElement;

  for (const [key, val] of Object.entries(colorsState.palette)) {
    root.style.setProperty(`--color-${key}`, val);
  }
  root.style.setProperty(`--color-foreground`, colorsState.palette.ok100);
  root.style.setProperty(`--color-background`, colorsState.palette.ok300);
}
