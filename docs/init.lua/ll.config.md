# ll.config

Function that returns the static config struct.

```lua

ll.config({

  storage = {
    -- path to library root containing all your albums
    library = "",
    -- path to .env file leland will load for actions execution
    environment = "",
  },

  manifest = {
    audio_files = { "flac" },
  },

  compiler = {
  },

})
```

