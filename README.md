## Eluxum

`...postum avion, wizard kiss and all be gone...`

Plaintext driven, local-first album centric library manager and web-based `mpd` client.

Centers primarily on collection of albums, uses palintext user-mutable `metadata.toml` files in each album folder root as SSOT for album metadata.

Uses compiler logic to derive both machine-readable `metadata.lock.json` and human-readable `metadata.lock.toml` files (located in album root) to fully describe state of any given album.

Compiler logic divides all `key: value` pairs into 4 distinct classes:
* Album scope `TAGS`
* Album scope `helpers`
* Track scope `TAGS`
* Track scope `helpers`

Album scope keys are applied to all existing tracks within single album instance.
Track scope keys are applied to each of tracks individually

`TAGS` are namespaced to uppercase and are used to serve standard Vorbis Dictionary keys.
`helpers` are namespaced to lowercase and are used to serve any computable data to client user interface.
