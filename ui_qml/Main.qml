import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: window
    width: 1280
    height: 800
    visible: true
    title: "Vellum"

    Theme { id: theme }

    background: Rectangle {
        color: theme.backgroundMain
    }

    AlbumGrid {
        anchors.fill: parent
    }
}
