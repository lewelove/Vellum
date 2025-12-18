import uvicorn
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/api/library")
def get_library():
    albums = []
    colors = ["#EA4335", "#34A853", "#FBBC04", "#4285F4", "#A142F4", "#4285F4"]
    for i in range(1, 51):
        c_index = (i - 1) % len(colors)
        albums.append({
            "id": i,
            "title": f"ALBUM {i}",
            "artist": f"ARTIST {i}",
            "color": colors[c_index],
            "tracks": ["Track 1", "Track 2", "Track 3"]
        })
    return albums

@app.post("/api/play/{album_id}")
def play_album(album_id: int):
    print(f"I am now playing album {album_id}")
    return {"status": "success", "message": f"Playing album {album_id}"}

if __name__ == "__main__":
    # ARCHITECTURAL FIX: 
    # Since this file is 'main.py', the import string must be "main:app"
    uvicorn.run("main:app", host="127.0.0.1", port=8000, reload=True)
