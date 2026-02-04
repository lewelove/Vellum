import QtQuick

QtObject {
    // Basic Palette
    readonly property color backgroundMain: "#323232"
    readonly property color backgroundDrawer: "#242424"
    readonly property color textMain: "#FFFFFF"
    readonly property color textMuted: "#CCCCCC"
    
    // Grid Constants
    readonly property int coverSize: 200
    readonly property int gapX: 24
    readonly property int gapY: 12
    readonly property int textGapMain: 8
    readonly property int textGapLesser: 2

    // Typography (Match Svelte line-heights)
    readonly property int fontSizeTitle: 14
    readonly property int lineHeightTitle: 18
    readonly property int fontSizeArtist: 12
    readonly property int lineHeightArtist: 16

    // SSOT: Calculated Row Height 
    // Mirrors: gapY + coverSize + textGapMain + lhTitle + gapLesser + lhArtist
    readonly property int rowHeight: gapY + coverSize + textGapMain + lineHeightTitle + textGapLesser + lineHeightArtist
    
    // SSOT: Cell Width
    readonly property int cellWidth: coverSize + gapX
}
