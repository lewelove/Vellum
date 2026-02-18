/**
 * WebSocket Connection Logic
 * 
 * Logic Change:
 * The connection URL is modified to bypass the Vite development proxy (port 5173) 
 * and connect directly to the Rust backend (port 8000). This circumvents a 
 * known issue in Bun's Vite proxy implementation where large JSON payloads 
 * (such as a library of ~800+ albums) are truncated or dropped, causing 
 * the UI to fail during the INIT phase.
 */
export function connectSocket(onOpen, onMessage) {
  // Direct connection to the backend to ensure payload integrity
  const protocol = 'ws:';
  const host = '127.0.0.1:8000'; 
  const url = `${protocol}//${host}/ws`;

  let socket = new WebSocket(url);

  socket.onopen = () => {
    console.log("Vellum WebSocket: Connected to backend");
    if (onOpen) onOpen();
  };

  socket.onmessage = (event) => {
    if (onMessage) onMessage(event);
  };

  socket.onclose = () => {
    console.log("Vellum WebSocket: Disconnected. Reconnecting...");
    setTimeout(() => {
      connectSocket(onOpen, onMessage);
    }, 2000);
  };

  socket.onerror = (err) => {
    console.error("Vellum WebSocket: Error", err);
  };

  return socket;
}

export async function playAlbum(id, offset = 0) {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/play/${encodedId}?offset=${offset}`, { method: "POST" });
  return await response.json();
}

export async function playDisc(id, discNumber) {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/play-disc/${encodedId}?disc=${discNumber}`, { method: "POST" });
  return await response.json();
}

export async function queueAlbum(id) {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/queue/${encodedId}`, { method: "POST" });
  return await response.json();
}

export async function openAlbumFolder(id) {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/open/${encodedId}`, { method: "POST" });
  return await response.json();
}
