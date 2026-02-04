import sys
import os
from pathlib import Path
from PySide6.QtGui import QGuiApplication
from PySide6.QtQml import QQmlApplicationEngine
from PySide6.QtCore import QObject, Slot

class VellumBridge(QObject):
    """Bridge for communication between Python and QML."""
    def __init__(self):
        super().__init__()

    @Slot(str)
    def log(self, message):
        print(f"[QML] {message}")

def main():
    # Fix for high-DPI scaling
    os.environ["QT_AUTO_SCREEN_SCALE_FACTOR"] = "1"
    
    app = QGuiApplication(sys.argv)
    app.setOrganizationName("Vellum")
    app.setApplicationName("Vellum-QML")

    engine = QQmlApplicationEngine()
    
    # Expose the bridge
    bridge = VellumBridge()
    engine.rootContext().setContextProperty("bridge", bridge)

    # Load the UI
    qml_file = Path(__file__).parent / "Main.qml"
    engine.load(str(qml_file))

    if not engine.rootObjects():
        sys.exit(-1)

    print("Vellum QML UI Started.")
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
