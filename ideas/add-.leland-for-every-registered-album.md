# add .dale for every registered album

the idea is to transition registered album to inclusion of `.dale` folder inside it

`.dale` holds:
- compiled album.lock.json and album.lock.jsonb
- generated local.toml

any `.toml` manifest placed in `.dale` instantly registered and merged into lock
