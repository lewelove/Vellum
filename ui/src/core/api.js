// The API layer is now minimal. Logic moved to Client.

export async function getLibraryArtifact() {
  // Fetches the static JSON database
  const response = await fetch("/library.json");
  if (!response.ok) throw new Error("Failed to load library artifact");
  return await response.json();
}

export async function playAlbum(id) {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/play/${encodedId}`, { method: "POST" });
  return await response.json();
}
