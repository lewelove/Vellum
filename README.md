## Vellum

Plain-text driven, album centric library gallery and web-based `mpd` client.

---

Uses plain-text user-mutable `metadata.toml` sidecar files in each album folder root to describe metadata state of any given album. Compiles `metadata.toml` and audio file physics into `metadata.json.lock`, which allows to programmatically derive and set any information from album / tracks related data at compile time.

Separation of any user-mutable metadata from audio binary itself to plain-text files allows to:
- version control them easily with git
- free yourself from having to edit binary headers

Compiler logic divides all `key = value` pairs into 4 distinct classes:
* Album scope `TAGS`
* Album scope `helpers`
* Track scope `TAGS`
* Track scope `helpers`

Album scope keys are applied to all existing tracks within single album instance.
Track scope keys are applied to each of tracks individually

`TAGS` are namespaced to uppercase and are used to serve standard Vorbis Dictionary keys.
`helpers` are namespaced to lowercase and are used to serve any additional computable at compile time data to client user interface.

