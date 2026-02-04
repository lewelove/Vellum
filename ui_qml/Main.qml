import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: window
    width: 1280
    height: 800
    visible: true
    title: "Vellum Native"

    background: Rectangle {
        color: "#111111"
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Sidebar Area
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

                Button {
                    text: "Media Library"
                    flat: true
                    onClicked: bridge.log("Library Clicked")
                }
            }
        }

        // Content Area
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "transparent"

            Text {
                anchors.centerIn: parent
                text: "QML Grid Implementation Pending..."
                color: "#444"
                font.pixelSize: 24
            }
        }
    }

    Component.onCompleted: {
        bridge.log("Window Initialized");
    }
}
