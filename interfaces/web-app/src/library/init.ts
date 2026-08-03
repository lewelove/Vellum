import { sync } from "./sync.svelte.ts";
import { collection } from "./collection.svelte.ts";
import { prewarmer } from "./prewarmer.svelte.ts";
import { updateConfig } from "../config.svelte.ts";
import { applyColors } from "../colors.svelte.ts";

export function initApp() {
  prewarmer;

  sync.connect();

  fetch("/api/interfaces/default/config")
    .then((res) => res.json())
    .then((data) => {
      collection.config = { ...collection.config, ...data };
      updateConfig(data);
      applyColors(data);
    })
    .catch(() => {
      fetch("/api/interfaces/web-app/config")
        .then((res) => res.json())
        .then((data) => {
          collection.config = { ...collection.config, ...data };
          updateConfig(data);
          applyColors(data);
        })
        .catch((e) => console.error(e));
    });
}
