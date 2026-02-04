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
        color: "#111111"
    }

    AlbumGrid {
        anchors.fill: parent
    }
}
