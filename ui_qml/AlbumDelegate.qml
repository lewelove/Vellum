import QtQuick
import QtQuick.Layouts

Item {
    id: root
    width: theme.cellWidth
    height: theme.rowHeight - theme.gapY

    property var album: albumData
    Theme { id: theme }

    Column {
        width: theme.coverSize
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: theme.gapY
        spacing: theme.textGapMain

        // Cover Art
        Rectangle {
            width: theme.coverSize
            height: theme.coverSize
            color: theme.placeholderColor
            
            Image {
                anchors.fill: parent
                source: album.cover_hash ? "http://127.0.0.1:8000/api/covers/" + album.cover_hash + ".png" : ""
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
        }

        // Text Block
        Column {
            width: parent.width
            spacing: theme.textGapLesser

            Text {
                width: parent.width
                height: theme.lineHeightTitle
                text: album.ALBUM || ""
                color: theme.textMain
                font.pixelSize: theme.fontSizeTitle
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
            }

            Text {
                width: parent.width
                height: theme.lineHeightArtist
                text: album.ALBUMARTIST || ""
                color: theme.textMuted
                font.pixelSize: theme.fontSizeArtist
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
            }
        }
    }
}
