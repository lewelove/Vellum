import QtQuick
import QtQuick.Layouts

Item {
    id: root
    width: theme.coverSize
    height: theme.coverSize + theme.textGapMain + 40 // Approx text height

    property var album: albumData
    Theme { id: theme }

    Column {
        anchors.fill: parent
        spacing: theme.textGapMain

        Rectangle {
            width: theme.coverSize
            height: theme.coverSize
            color: "#323232"
            
            Image {
                anchors.fill: parent
                source: album.cover_hash ? "http://127.0.0.1:8000/api/covers/" + album.cover_hash + ".png" : ""
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
        }

        Column {
            width: parent.width
            spacing: theme.textGapLesser

            Text {
                width: parent.width
                text: album.ALBUM || ""
                color: theme.textMain
                font.pixelSize: theme.fontSizeTitle
                elide: Text.ElideRight
            }

            Text {
                width: parent.width
                text: album.ALBUMARTIST || ""
                color: theme.textMuted
                font.pixelSize: theme.fontSizeArtist
                elide: Text.ElideRight
            }
        }
    }
}
