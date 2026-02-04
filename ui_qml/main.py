import sys, os, json, threading, websocket
from pathlib import Path
from PySide6.QtGui import QGuiApplication
from PySide6.QtQml import QQmlApplicationEngine
from PySide6.QtCore import QObject, Slot, Signal, QAbstractListModel, Qt

class AlbumModel(QAbstractListModel):
    DataRole = Qt.UserRole + 1
    def __init__(self):
        super().__init__()
        self._albums = []
    def rowCount(self, parent=None): return len(self._albums)
    def data(self, index, role=Qt.DisplayRole):
        if 0 <= index.row() < len(self._albums) and role == self.DataRole:
            return self._albums[index.row()]
        return None
    def roleNames(self): return {self.DataRole: b"albumData"}
    
    @Slot(list)
    def update_all(self, data):
        self.beginResetModel()
        self._albums = data
        self.endResetModel()

class VellumBridge(QObject):
    data_received = Signal(list)
    def __init__(self):
        super().__init__()
        threading.Thread(target=self._ws_thread, daemon=True).start()

    def _ws_thread(self):
        ws = websocket.WebSocketApp("ws://127.0.0.1:8000/ws", on_message=self._on_msg)
        ws.run_forever()

    def _on_msg(self, ws, message):
        data = json.loads(message)
        if data["type"] == "INIT":
            self.data_received.emit(data["data"])

def main():
    app = QGuiApplication(sys.argv)
    engine = QQmlApplicationEngine()
    bridge = VellumBridge()
    model = AlbumModel()
    bridge.data_received.connect(model.update_all)
    
    engine.rootContext().setContextProperty("bridge", bridge)
    engine.rootContext().setContextProperty("albumModel", model)
    engine.load(str(Path(__file__).parent / "Main.qml"))
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
