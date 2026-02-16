import QtQuick
import QtQuick.Controls

Item {
    id: control
    clip: true

    Theme { id: theme }

    readonly property int calculatedCols: Math.max(1, Math.floor(control.width / theme.cellWidth))
    readonly property real gridContentWidth: calculatedCols * theme.cellWidth

    Scroll {
        id: scrollEngine
        rowCount: grid.count
        columns: calculatedCols
        rowHeight: theme.rowHeight
        viewportHeight: control.height
        anchors.fill: parent
        
        Component.onCompleted: forceActiveFocus()
    }

    GridView {
        id: grid
        width: gridContentWidth
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        
        cellWidth: theme.cellWidth
        cellHeight: theme.rowHeight

        model: albumModel
        delegate: AlbumDelegate {}

        interactive: false
        contentY: scrollEngine.currentY
        
        cacheBuffer: theme.rowHeight * 2
    }

    Rectangle {
        id: scrollbar
        anchors.right: parent.right
        anchors.rightMargin: 4
        
        y: scrollEngine.maxSlots > 0 
           ? (scrollEngine.targetSlot / scrollEngine.maxSlots) * (control.height - height) 
           : 0

        width: 3
        height: Math.max(40, (control.height / Math.max(control.height, grid.contentHeight)) * control.height)
        color: theme.scrollbarColor
        opacity: 0.15
        radius: 1.5
        visible: scrollEngine.maxSlots > 0
    }
}
