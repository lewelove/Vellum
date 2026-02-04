import QtQuick
import QtQuick.Controls

ScrollView {
    id: control
    contentWidth: availableWidth
    clip: true

    Theme { id: theme }

    GridView {
        id: grid
        anchors.fill: parent
        anchors.leftMargin: 40
        anchors.rightMargin: 40
        anchors.topMargin: theme.gapY

        model: albumModel
        cellWidth: theme.coverSize + theme.gapX
        cellHeight: theme.coverSize + 60 // cover + gaps + text
        
        delegate: AlbumDelegate {}

        // Simple animation to match the smooth feel of the Svelte grid
        add: Transition {
            NumberAnimation { property: "opacity"; from: 0; to: 1.0; duration: 200 }
        }
    }
}
