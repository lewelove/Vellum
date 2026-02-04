import QtQuick
import QtQuick.Layouts

Item {
    id: root
    width: theme.coverSize
    height: theme.rowHeight - theme.gapY // The delegate handles its own internal spacing

    property var album: albumData
    Theme { id: theme }

    Column {
        anchors.fill: parent
        anchors.topMargin: theme.gapY // Consistent with Svelte's row padding
        spacing: theme.textGapMain

        // Cover Art
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
