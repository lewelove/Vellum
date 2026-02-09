import orjson
import subprocess
from fastapi import APIRouter, HTTPException, WebSocket, WebSocketDisconnect, Request
from fastapi.responses import FileResponse
from . import config
from .library import STATE
from .mpd_engine import play_album_logic, enqueue_album_logic, play_disc_logic

class ConnectionManager:
    def __init__(self): 
        self.active_connections = []
        
    async def connect(self, ws: WebSocket): 
        await ws.accept()
        self.active_connections.append(ws)
        
    def disconnect(self, ws: WebSocket): 
        if ws in self.active_connections: 
            self.active_connections.remove(ws)
            
    async def broadcast_bytes(self, data: bytes):
        for c in self.active_connections:
            try: 
                await c.send_bytes(data)
            except: 
                pass

manager = ConnectionManager()
router = APIRouter()

@router.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await manager.connect(websocket)
    try:
        init_payload = {
            "type": "INIT", 
            "data": STATE.albums,
            "ui_state": config.UI_STATE
        }
        await websocket.send_bytes(orjson.dumps(init_payload))
        while True: 
            await websocket.receive_text()
    except WebSocketDisconnect: 
        manager.disconnect(websocket)

@router.post("/api/state")
async def update_state(request: Request):
    try:
        data = await request.json()
        config.UI_STATE.update(data)
        config.save_ui_state()
        return {"status": "saved"}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/api/internal/reset")
async def trigger_full_reset():
    try:
        config.load_config()
        await STATE.initialize()
        
        payload = {
            "type": "INIT", 
            "data": STATE.albums,
            "ui_state": config.UI_STATE
        }
        await manager.broadcast_bytes(orjson.dumps(payload))
        return {"status": "reset_complete"}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.post("/api/internal/reload")
async def trigger_reload(path: str):
    updated = STATE.update_album(path)
    if updated:
        await manager.broadcast_bytes(
            orjson.dumps({"type": "UPDATE", "id": updated["id"], "payload": updated})
        )
        return {"status": "reloaded"}
    raise HTTPException(status_code=404)

@router.get("/api/covers/{cover_hash}.png")
def get_cover_thumbnail(cover_hash: str):
    if not config.THUMBNAIL_ROOT:
        raise HTTPException(status_code=404)
    path = (config.THUMBNAIL_ROOT / f"{cover_hash}.png").resolve()
    if not path.exists(): 
        raise HTTPException(status_code=404)
    return FileResponse(
        path, 
        headers={
            "Cache-Control": "public, max-age=31536000, immutable"
        }
    )

@router.get("/api/assets/{album_id:path}/cover")
def get_album_cover(album_id: str):
    album = STATE.album_map.get(album_id)
    if not album or not album.get("cover_path") or album.get("cover_path") == "default_cover.png":
        raise HTTPException(status_code=404)
    
    if not config.LIBRARY_ROOT:
        raise HTTPException(status_code=500, detail="Library root not configured")

    path = (config.LIBRARY_ROOT / album_id / album["cover_path"]).resolve()
    if not path.exists(): 
        raise HTTPException(status_code=404)
    return FileResponse(
        path,
        headers={
            "Cache-Control": "public, max-age=31536000, immutable"
        }
    )

@router.post("/api/play/{album_id:path}")
def play_album(album_id: str, offset: int = 0):
    success = play_album_logic(album_id, offset)
    if not success:
        raise HTTPException(status_code=404, detail="Could not play album")
    return {"status": "ok"}

@router.post("/api/play-disc/{album_id:path}")
def play_disc(album_id: str, disc: str):
    success = play_disc_logic(album_id, disc)
    if not success:
        raise HTTPException(status_code=404, detail="Could not play disc")
    return {"status": "ok"}

@router.post("/api/queue/{album_id:path}")
def queue_album(album_id: str):
    success = enqueue_album_logic(album_id)
    if not success:
        raise HTTPException(status_code=404, detail="Could not queue album")
    return {"status": "ok"}

@router.post("/api/open/{album_id:path}")
def open_album_folder(album_id: str):
    if not config.LIBRARY_ROOT:
        raise HTTPException(status_code=500, detail="Library root not configured")
    
    path = (config.LIBRARY_ROOT / album_id).resolve()
    if not path.exists():
        raise HTTPException(status_code=404, detail="Folder not found")

    try:
        subprocess.Popen(["xdg-open", str(path)])
        return {"status": "ok"}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
