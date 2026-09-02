# theme.toml

This manifest provides the colors for album to be displayed in the UI. Values are always must be a valid CSS HEX color strings.

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

### colors.foreground

Determines the color of all foreground elements used inside album display panels in QueueView.

### colors.background

Determines the colors that are fed into a background shader.

### album.lock.json

```jsonc
{
  "album": {
    "colors": {
      "foreground": "", // single HEX string
      "background": [ "" ] // array of HEX strings
    }
  }
}
```
