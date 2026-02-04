import QtQuick

QtObject {
    // Basic Palette
    readonly property color backgroundMain: "#323232"
    readonly property color backgroundDrawer: "#242424"
    readonly property color textMain: "#FFFFFF"
    readonly property color textMuted: "#CCCCCC"
    
    // UI Elements
    readonly property color scrollbarColor: textMain
    readonly property color placeholderColor: "#2A2A2A"

    // Grid Constants
    readonly property int coverSize: 200
    readonly property int gapX: 24
    readonly property int gapY: 12
    readonly property int textGapMain: 8
    readonly property int textGapLesser: 2

    // Typography
    readonly property int fontSizeTitle: 14
    readonly property int lineHeightTitle: 18
    readonly property int fontSizeArtist: 12
    readonly property int lineHeightArtist: 16

    // Calculated Constants
    readonly property int rowHeight: gapY + coverSize + textGapMain + lineHeightTitle + textGapLesser + lineHeightArtist
    readonly property int cellWidth: coverSize + gapX
}
