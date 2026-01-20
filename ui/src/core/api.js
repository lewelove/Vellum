// The API layer is now minimal. Logic moved to Client.

export async function getLibraryArtifact() {
  // Fetches the static JSON database
  // We append a timestamp to bypass browser caching
  const bust = Date.now();
  const response = await fetch(`/library.json?t=${bust}`);
  
  if (!response.ok) throw new Error("Failed to load library artifact");
  return await response.json();
}

export async function playAlbum(id) {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/play/${encodedId}`, { method: "POST" });
  return await response.json();
}
