export const nav = $state({
  activeTab: 'home'
});

export async function setTab(tab) {
  if (nav.activeTab === tab) return;
  nav.activeTab = tab;
  
  const { library } = await import("./library.svelte.js");
  library.persistState();
}
