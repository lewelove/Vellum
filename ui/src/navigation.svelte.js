export const nav = $state({
  activeTab: 'home'
});

export function setTab(tab) {
  nav.activeTab = tab;
}
