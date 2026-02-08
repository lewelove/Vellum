export function connectSocket(onOpen, onMessage) {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = window.location.host; 
  const url = `${protocol}//${host}/ws`;

  let socket = new WebSocket(url);

  socket.onopen = () => {
    console.log("Live Lake Connected");
    if (onOpen) onOpen();
  };

  socket.onmessage = (event) => {
    if (onMessage) onMessage(event);
  };

  socket.onclose = () => {
    console.log("Live Lake Disconnected. Reconnecting in 2s...");
    setTimeout(() => {
      connectSocket(onOpen, onMessage);
    }, 2000);
  };

  socket.onerror = (err) => {
    console.error("Socket error", err);
  };

  return socket;
}

export async function playAlbum(id, offset = 0) {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/play/${encodedId}?offset=${offset}`, { method: "POST" });
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
