export async function getLibrary() {
  const response = await fetch("/api/library");
  if (!response.ok) throw new Error("Failed to fetch library");
  return await response.json();
}

export async function playAlbum(id) {
  const response = await fetch(`/api/play/${id}`, { method: "POST" });
  return await response.json();
}
