from fastapi import WebSocket

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
