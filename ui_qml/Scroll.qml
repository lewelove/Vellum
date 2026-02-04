import QtQuick

Item {
    id: root
    
    // -------------------------------------------------------------------------
    // External API
    // -------------------------------------------------------------------------
    property int rowCount: 0
    property int columns: 1
    property real rowHeight: 1 // Default to 1 to prevent division by zero
    property real viewportHeight: 0
    
    // Output for GridView
    property real currentY: 0

    // -------------------------------------------------------------------------
    // Configuration
    // -------------------------------------------------------------------------
    readonly property real damping: 0.16
    readonly property real keySpeed: 0.10
    readonly property int wheelThreshold: 40
    
    // -------------------------------------------------------------------------
    // Internal State
    // -------------------------------------------------------------------------
    property real targetSlot: 0
    property real maxSlots: 0
    property real wheelAccumulator: 0
    
    // Input Flags
    property bool k_up: false
    property bool k_down: false
    property bool k_j: false
    property bool k_k: false

    // -------------------------------------------------------------------------
    // bounds Safety
    // -------------------------------------------------------------------------
    function recalcBounds() {
        if (rowHeight <= 1) {
            maxSlots = 0;
            return;
        }
        let totalRows = Math.ceil(rowCount / columns);
        let visibleRows = Math.floor(viewportHeight / rowHeight);
        let calculatedMax = Math.max(0, totalRows - visibleRows);
        
        // Safety clamp if window resized or data changed
        if (targetSlot > calculatedMax) targetSlot = calculatedMax;
        
        maxSlots = calculatedMax;
    }

    onRowCountChanged: recalcBounds()
    onColumnsChanged: recalcBounds()
    onRowHeightChanged: recalcBounds()
    onViewportHeightChanged: recalcBounds()

    // -------------------------------------------------------------------------
    // Input Handling
    // -------------------------------------------------------------------------
    focus: true // Critical for Keys
    Keys.enabled: true
    
    Keys.onPressed: (event) => {
        if (event.isAutoRepeat) return;
        switch (event.key) {
            case Qt.Key_J:    k_j = true; event.accepted = true; break;
            case Qt.Key_K:    k_k = true; event.accepted = true; break;
            case Qt.Key_Down: k_down = true; event.accepted = true; break;
            case Qt.Key_Up:   k_up = true; event.accepted = true; break;
        }
    }

    Keys.onReleased: (event) => {
        if (event.isAutoRepeat) return;
        switch (event.key) {
            case Qt.Key_J:    k_j = false; event.accepted = true; break;
            case Qt.Key_K:    k_k = false; event.accepted = true; break;
            case Qt.Key_Down: k_down = false; event.accepted = true; break;
            case Qt.Key_Up:   k_up = false; event.accepted = true; break;
        }
    }

    WheelHandler {
        acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
        onWheel: (event) => {
            // angleDelta.y > 0 = Scroll UP (Wheel pushed away) -> We want to go to LOWER indices
            // angleDelta.y < 0 = Scroll DOWN (Wheel pulled close) -> We want to go to HIGHER indices
            
            // Invert logic: Scroll Down (negative y) should ADD to accumulator to go down
            root.wheelAccumulator -= event.angleDelta.y
            
            if (Math.abs(root.wheelAccumulator) >= root.wheelThreshold) {
                let direction = root.wheelAccumulator > 0 ? 1 : -1
                let base = Math.round(root.targetSlot)
                
                // Clamp immediately to prevent "space" launch
                let next = Math.max(0, Math.min(base + direction, root.maxSlots))
                
                root.targetSlot = next
                root.wheelAccumulator = 0
            }
            event.accepted = true
        }
    }

    // -------------------------------------------------------------------------
    // Physics Loop (Svelte Replica)
    // -------------------------------------------------------------------------
    FrameAnimation {
        running: true
        onTriggered: {
            // 1. Process Continuous Key Input
            let delta = 0;
            if (root.k_down || root.k_j) delta += root.keySpeed;
            if (root.k_up || root.k_k)   delta -= root.keySpeed;
            
            if (delta !== 0) {
                root.targetSlot = Math.max(0, Math.min(root.targetSlot + delta, root.maxSlots));
            }

            // 2. Damping Physics (Asymptotic approach)
            let targetPixelY = root.targetSlot * root.rowHeight;
            let diff = targetPixelY - root.currentY;
            
            // Deadzone to prevent micro-jitter at rest
            if (Math.abs(diff) < 0.1) {
                root.currentY = targetPixelY;
            } else {
                root.currentY += diff * root.damping;
            }
        }
    }
}
