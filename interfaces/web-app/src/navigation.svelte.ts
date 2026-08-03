import { view } from "./library/view.svelte.ts";

export const nav = $state<{ activeTab: string }>({
  activeTab: 'home'
});

export function setTab(tab: string) {
  if (nav.activeTab === tab) return;
  nav.activeTab = tab;

  view.refreshView(false);
  view.persistState();
}
