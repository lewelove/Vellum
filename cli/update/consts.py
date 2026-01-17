# TAGS (UPPERCASE)
ALBUM_TAGS = [
    "ALBUMARTIST",
    "ALBUM",
    "DATE",
    "GENRE",
    "TOTALTRACKS",
    "TOTALDISCS",
    "ORIGINAL_YYYY_MM",
    "ORIGINAL_YEAR",
    "ORIGINAL_DATE",
    "RELEASE_YYYY_MM",
    "RELEASE_YEAR",
    "RELEASE_DATE",
    "COUNTRY",
    "LABEL",
    "CATALOGNUMBER",
    "MEDIA",
    "COMMENT",
    "UNIX_ADDED_PRIMARY",
    "UNIX_ADDED_LOCAL",
    "UNIX_ADDED_APPLEMUSIC",
    "UNIX_ADDED_YOUTUBE",
    "CUSTOM_ID",
    "CUSTOM_ALBUMARTIST",
    "CUSTOM_STRING",
    "OLD_COMMENT",
    "DISCOGS_URL",
    "MUSICBRAINZ_URL",
    "CTDBID",
    "ACCURIPID",
    "DISCID",
    "MUSICBRAINZ_ALBUMARTISTID",
    "MUSICBRAINZ_RELEASEGROUPID",
    "MUSICBRAINZ_ALBUMID"
]

TRACK_TAGS = [
    "TITLE",
    "ARTIST",
    "TRACKNUMBER",
    "DISCNUMBER",
    "MUSICBRAINZ_ARTISTID",
    "MUSICBRAINZ_RELEASETRACKID",
    "MUSICBRAINZ_TRACKID"
]

# HELPERS (snake_case)
ALBUM_HELPERS = [
    "album_root_path",
    "metadata_toml_hash",
    "metadata_toml_mtime",
    "unix_added",
    "date_added",
    # "album_duration_in_ms",
    "album_duration_time",
    "cover_path",
    # "cover_path_absolute",
    "cover_byte_size",
    "cover_mtime"
]

TRACK_HELPERS = [
    "track_path",
    "track_path_absolute",
    "track_mtime",
    "track_size",
    "lyrics_path",
    "lyrics_path_absolute",
    "encoding",
    "bits_per_sample",
    "channels",
    "sample_rate",
    "track_duration_in_samples",
    "track_duration_in_ms",
    "track_duration_time"
]
