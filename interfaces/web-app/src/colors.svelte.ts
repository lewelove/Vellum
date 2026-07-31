export interface ColorsConfig {
  foreground: string;
  background: string;
}

export const colorsState = $state<{
  colors: ColorsConfig;
}>({
  colors: {
    foreground: "oklch(1.00 0 0)",
    background: "oklch(0.26 0 0)"
  }
});

export function applyColors(configData: any) {
  if (!configData) return;

  if (configData.colors) {
    Object.assign(colorsState.colors, configData.colors);
  }

  if (typeof document === "undefined") return;
  const root = document.documentElement;

  for (const [key, val] of Object.entries(colorsState.colors)) {
    root.style.setProperty(`--color-${key}`, val);
  }
}
