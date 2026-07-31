import { applyColors } from "./colors.svelte.ts";

export const config = $state({
  shader: {
    order: "original",
    speed: 1.7,
    zoom: 0.3,
    blur: 0.8,
    grain: 0.1,
    equalize: 1.0,
  },
  colors: {
    foreground: "oklch(1.00 0 0)",
    background: "oklch(0.26 0 0)"
  },
  album_grid: {
    spacing: { x: 20, y: 16, top: 20 },
    album_card: {
      cover: { size: 200, filter: "lanczos" },
      text: {
        enable: true,
        title: { size: 14 },
        albumartist: { size: 12 },
        spacing: { top: 11, middle: 2 },
      }
    }
  }
});

export function updateConfig(newData: any) {
  if (!newData) return;

  if (newData.colors) {
    Object.assign(config.colors, newData.colors);
    applyColors(newData);
  }

  if (newData.shader) {
    Object.assign(config.shader, newData.shader);
  }

  if (newData.album_grid) {
    if (newData.album_grid.spacing) {
      Object.assign(config.album_grid.spacing, newData.album_grid.spacing);
    }

    if (newData.album_grid.album_card) {
      if (newData.album_grid.album_card.cover) {
        Object.assign(config.album_grid.album_card.cover, newData.album_grid.album_card.cover);
      }

      if (newData.album_grid.album_card.text) {
        if (newData.album_grid.album_card.text.title) {
          Object.assign(config.album_grid.album_card.text.title, newData.album_grid.album_card.text.title);
        }
        if (newData.album_grid.album_card.text.albumartist) {
          Object.assign(config.album_grid.album_card.text.albumartist, newData.album_grid.album_card.text.albumartist);
        }
        if (newData.album_grid.album_card.text.spacing) {
          Object.assign(config.album_grid.album_card.text.spacing, newData.album_grid.album_card.text.spacing);
        }
        if (newData.album_grid.album_card.text.enable !== undefined) {
          config.album_grid.album_card.text.enable = newData.album_grid.album_card.text.enable;
        }
      }
    }
  }
}
