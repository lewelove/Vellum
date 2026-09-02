# musicbrainz_search

This action is used to construct and open search query link for release-groups from album locks or custom arguments

## How It Works

If no arguments provided:
- Read all albums from intermediary
- For each of them construct `artist:"{album.albumartist}" AND releasegroup:"{album.album}"` query string
- For each open `https://musicbrainz.org/search?type=release_group&method=advanced&query={query_string}` in xdg-open

If argument is `discogs.com/release` or `discogs.com/master`:
- Collapse `release` to `master` (if can't use raw `release` data)
- Read `albumartist` and `album` from resolved payload
- Construct MB query and open
