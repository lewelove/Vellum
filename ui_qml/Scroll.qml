import QtQuick

Item {
    id: root
    anchors.fill: parent

    property int rowCount: 0
    property int columns: 1
    property real rowHeight: 0
    property real viewportHeight: 0

    property int targetSlot: 0
    readonly property int maxSlots: Math.max(0, Math.ceil(rowCount / columns) - Math.floor(viewportHeight / rowHeight))

    property real currentY: 0

    Behavior on currentY {
        NumberAnimation {
            duration: 750
            easing.type: Easing.OutCubic
        }
    }

    onTargetSlotChanged: {
        currentY = targetSlot * rowHeight
    }

    focus: true
    Keys.onPressed: (event) => {
        if (event.key === Qt.Key_J || event.key === Qt.Key_Down) {
            targetSlot = Math.min(maxSlots, targetSlot + 1)
            event.accepted = true
        }
        if (event.key === Qt.Key_K || event.key === Qt.Key_Up) {
            targetSlot = Math.max(0, targetSlot - 1)
            event.accepted = true
        }
    }

    WheelHandler {
        acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
        orientation: Qt.Vertical
        onWheel: (event) => {
            if (event.angleDelta.y < 0) {
                root.targetSlot = Math.min(root.maxSlots, root.targetSlot + 1)
            } else {
                root.targetSlot = Math.max(0, root.targetSlot - 1)
            }
        }
    }
}
