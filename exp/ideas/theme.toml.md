# theme.toml

This manifest provides the colors for album to be displayed in the UI. Values are always must be a CSS compatible strings, that compile into an `oklab()` values.

```toml
[album.colors]

# A single string
foreground = ""
# Either an array or a string
background = []

[album.fonts]
main = ""
monospace = ""
```

## Specifications

### foreground

Determines the color of all foreground elements used inside album display panels. Primarily the QueueView.

### background

Determines the colors that are fed into a background shader.

