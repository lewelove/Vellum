import QtQuick
import QtQuick.Controls

Item {
    id: control
    clip: true

    Theme { id: theme }

    readonly property int calculatedCols: Math.max(1, Math.floor(control.width / theme.cellWidth))
    readonly property real gridContentWidth: (calculatedCols * theme.cellWidth) - theme.gapX

    Scroll {
        id: scrollEngine
        rowCount: grid.count
        columns: calculatedCols
        rowHeight: theme.rowHeight
        viewportHeight: control.height
        Component.onCompleted: forceActiveFocus()
    }

    GridView {
        id: grid
        height: parent.height
        width: gridContentWidth
        anchors.horizontalCenter: parent.horizontalCenter
        
        cellWidth: theme.cellWidth
        cellHeight: theme.rowHeight

        model: albumModel
        delegate: AlbumDelegate {}

        interactive: false
        contentY: scrollEngine.currentY

        add: Transition {
            NumberAnimation { property: "opacity"; from: 0; to: 1.0; duration: 200 }
        }
    }

    Rectangle {
        id: scrollbar
        anchors.right: parent.right
        anchors.rightMargin: 4
        y: scrollEngine.maxSlots > 0 ? (scrollEngine.targetSlot / scrollEngine.maxSlots) * (control.height - height) : 0
        width: 3
        height: Math.max(40, (control.height / Math.max(control.height, (scrollEngine.rowCount / calculatedCols) * theme.rowHeight)) * control.height)
        color: "white"
        opacity: 0.15
        radius: 1.5
        visible: scrollEngine.maxSlots > 0
    }
}
