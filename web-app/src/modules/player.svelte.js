export const player = $state({
  state: "stop",
  currentAlbumId: null,
  currentFile: null,
  title: "",
  artist: "",
  elapsed: 0,
  duration: 0,
  lastUpdated: 0,
  queue: []
});

export function updatePlayerState(data) {
  player.state = data.state;
  player.currentAlbumId = data.album_id;
  player.currentFile = data.file;
  player.title = data.title || "";
  player.artist = data.artist || "";
  player.elapsed = parseFloat(data.elapsed || 0);
  player.duration = parseFloat(data.duration || 0);
  player.lastUpdated = performance.now();
  
  if (data.queue) {
    player.queue = data.queue;
  }
}
