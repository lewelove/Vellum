import QtQuick

Item {
    id: root
    anchors.fill: parent

    readonly property real damping: 0.06
    readonly property int wheelThreshold: 40
    readonly property real keyScrollSpeed: 0.20

    property int rowCount: 0
    property int columns: 1
    property real rowHeight: 0
    property real viewportHeight: 0
    
    property real currentY: 0
    property real targetSlot: 0
    property real wheelAccumulator: 0
    
    readonly property real maxSlots: Math.max(0, Math.ceil(rowCount / columns) - Math.floor(viewportHeight / rowHeight))

    property var activeKeys: {
        "j": false,
        "k": false,
        "up": false,
        "down": false
    }

    FrameAnimation {
        running: true
        onTriggered: {
            let delta = 0
            if (root.activeKeys["j"] || root.activeKeys["down"]) delta += root.keyScrollSpeed
            if (root.activeKeys["k"] || root.activeKeys["up"]) delta -= root.keyScrollSpeed
            
            if (delta !== 0) {
                root.targetSlot = Math.max(0, Math.min(root.targetSlot + delta, root.maxSlots))
            }

            let targetY = root.targetSlot * root.rowHeight
            let diff = targetY - root.currentY
            
            if (Math.abs(diff) < 0.01) {
                root.currentY = targetY
            } else {
                root.currentY += (diff * root.damping)
            }
        }
    }

    focus: true
    Keys.enabled: true
    
    Keys.onPressed: (event) => {
        if (event.isAutoRepeat) return;
        if (event.key === Qt.Key_J) activeKeys["j"] = true;
        if (event.key === Qt.Key_K) activeKeys["k"] = true;
        if (event.key === Qt.Key_Down) activeKeys["down"] = true;
        if (event.key === Qt.Key_Up) activeKeys["up"] = true;
    }

    Keys.onReleased: (event) => {
        if (event.isAutoRepeat) return;
        if (event.key === Qt.Key_J) activeKeys["j"] = false;
        if (event.key === Qt.Key_K) activeKeys["k"] = false;
        if (event.key === Qt.Key_Down) activeKeys["down"] = false;
        if (event.key === Qt.Key_Up) activeKeys["up"] = false;
    }

    WheelHandler {
        acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
        onWheel: (event) => {
            // angleDelta.y is usually 120 per notch. Svelte logic expects pixel-like delta.
            // Inverting to match web wheel behavior (down is positive deltaY).
            root.wheelAccumulator += -event.angleDelta.y
            
            if (Math.abs(root.wheelAccumulator) >= root.wheelThreshold) {
                let direction = root.wheelAccumulator > 0 ? 1 : -1
                let base = Math.round(root.targetSlot)
                root.targetSlot = Math.max(0, Math.min(base + direction, root.maxSlots))
                root.wheelAccumulator = 0
            }
        }
    }
}
