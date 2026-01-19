export async function getLibrary(params = {}) {
  const query = new URLSearchParams(params).toString();
  const url = query ? `/api/library?${query}` : "/api/library";
  
  const response = await fetch(url);
  if (!response.ok) throw new Error("Failed to fetch library");
  return await response.json();
}

export async function getCapabilities() {
  const response = await fetch("/api/capabilities");
  if (!response.ok) throw new Error("Failed to fetch capabilities");
  return await response.json();
}

export async function getSidebarGroup(key) {
  const response = await fetch(`/api/sidebar/${key}`);
  if (!response.ok) throw new Error(`Failed to fetch group ${key}`);
  return await response.json();
}

export async function playAlbum(id) {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/play/${encodedId}`, { method: "POST" });
  return await response.json();
}
