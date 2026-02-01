import asyncio
import uvicorn
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from . import config
from .library import STATE
from .mpd_engine import monitor_loop
from .api import router

@asynccontextmanager
async def lifespan(app: FastAPI):
    config.load_config()
    config.load_ui_state()
    
    if config.LIBRARY_ROOT:
        await STATE.initialize()
    
    monitor_task = asyncio.create_task(monitor_loop())
    
    yield
    
    monitor_task.cancel()
    try: 
        await asyncio.wait_for(monitor_task, timeout=2)
    except Exception: 
        pass

app = FastAPI(lifespan=lifespan)
app.add_middleware(
    CORSMiddleware, 
    allow_origins=["*"], 
    allow_credentials=True, 
    allow_methods=["*"], 
    allow_headers=["*"]
)

app.include_router(router)

if __name__ == "__main__":
    uvicorn.run("server.main:app", host="127.0.0.1", port=8000, log_level="info")
