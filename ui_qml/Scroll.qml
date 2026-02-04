import QtQuick

Item {
    id: root
    
    // -------------------------------------------------------------------------
    // External API
    // -------------------------------------------------------------------------
    property int rowCount: 0
    property int columns: 1
    property real rowHeight: 1 
    property real viewportHeight: 0
    
    // OUTPUT
    property real currentY: 0

    // -------------------------------------------------------------------------
    // Configuration
    // -------------------------------------------------------------------------
    // 0.15 = 15% closure per frame at 60fps.
    // Higher = Snappier. Lower = Heavier.
    readonly property real dampingFactor: 0.15 
    readonly property real keySpeed: 0.15
    readonly property int wheelThreshold: 40
    
    // -------------------------------------------------------------------------
    // Internal State
    // -------------------------------------------------------------------------
    property real targetSlot: 0
    property real maxSlots: 0
    property real wheelAccumulator: 0
    
    property bool k_up: false
    property bool k_down: false
    property bool k_j: false
    property bool k_k: false

    // -------------------------------------------------------------------------
    // Bounds Safety
    // -------------------------------------------------------------------------
    function recalcBounds() {
        if (rowHeight <= 1) {
            maxSlots = 0;
            return;
        }
        let totalRows = Math.ceil(rowCount / columns);
        let visibleRows = Math.floor(viewportHeight / rowHeight);
        let calculatedMax = Math.max(0, totalRows - visibleRows);
        
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
    focus: true
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
            // Negative angleDelta.y is usually "Scroll Down" (pull towards user)
            // We subtract it to make the Accumulator Positive when scrolling Down
            root.wheelAccumulator -= event.angleDelta.y
            
            if (Math.abs(root.wheelAccumulator) >= root.wheelThreshold) {
                let direction = root.wheelAccumulator > 0 ? 1 : -1
                
                // Snap to nearest integer slot before adding direction
                // This prevents getting stuck between slots
                let base = Math.round(root.targetSlot)
                let next = Math.max(0, Math.min(base + direction, root.maxSlots))
                
                root.targetSlot = next
                root.wheelAccumulator = 0
            }
            event.accepted = true
        }
    }

    // -------------------------------------------------------------------------
    // Physics Engine (Exponential Decay)
    // -------------------------------------------------------------------------
    FrameAnimation {
        running: true
        onTriggered: {
            // 1. Process Continuous Key Input
            let keyDelta = 0;
            if (root.k_down || root.k_j) keyDelta += root.keySpeed;
            if (root.k_up || root.k_k)   keyDelta -= root.keySpeed;
            
            if (keyDelta !== 0) {
                root.targetSlot = Math.max(0, Math.min(root.targetSlot + keyDelta, root.maxSlots));
            }

            // 2. Calculate Target Pixels
            let targetPixelY = root.targetSlot * root.rowHeight;
            let diff = targetPixelY - root.currentY;
            
            // 3. Apply Asymptotic Decay
            // Formula: velocity = distance * damping
            // Independent of frameTime to prevent lag-spikes from slowing scroll
            // dt is in seconds (e.g., 0.016)
            
            // If we are close enough, snap to prevent micro-jitter
            if (Math.abs(diff) < 0.5) {
                root.currentY = targetPixelY;
            } else {
                // Adjust damping for Delta Time so 60fps and 144fps feel the same
                // 60fps reference = 16.6ms
                let timeScale = (frameTime * 60); 
                root.currentY += diff * (root.dampingFactor * timeScale);
            }
        }
    }
}
