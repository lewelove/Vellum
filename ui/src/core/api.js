export async function getLibrary() {
  const response = await fetch("/api/library");
  if (!response.ok) throw new Error("Failed to fetch library");
  return await response.json();
}

export async function getAlbumTracks(id) {
  // id contains slashes, need to encode for URL safety
  // e.g. "Artist/Album" -> "Artist%2FAlbum"
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/album/${encodedId}`);
  if (!response.ok) throw new Error("Failed to fetch album tracks");
  return await response.json();
}

export async function playAlbum(id) {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/play/${encodedId}`, { method: "POST" });
  return await response.json();
}
