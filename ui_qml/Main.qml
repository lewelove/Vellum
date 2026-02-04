import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: window
    width: 1280
    height: 800
    visible: true
    title: "Vellum Native"

    Theme { id: theme }

    background: Rectangle {
        color: "#111111"
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 220
            color: "#181818"

            Column {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 15

                Text { 
                    text: "VELLUM"
                    color: "white"
                    font.pixelSize: 22
                    font.letterSpacing: 4
                }
                
                Rectangle { width: parent.width; height: 1; color: "#333" }
            }
        }

        AlbumGrid {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}
