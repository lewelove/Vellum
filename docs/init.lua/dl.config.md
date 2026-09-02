# dl.config

Function that returns the static config struct.

```lua

dl.config({

  storage = {
    -- path to library root containing all your albums
    library = "",
    -- path to .env file dale will load for actions execution
    environment = "",
  },

  manifest = {
    audio_files = { "flac" },
  },

  compiler = {
  },

})
```

