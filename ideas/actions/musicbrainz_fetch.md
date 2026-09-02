# musicbrainz_fetch

This action is used to save raw JSON response from MusicBrainz to the album directory for later consuming by the Dale compiler.

## CLI

`musicbrainz_fetch <URL> [options]`

## Options

- `<URL>` / `--url <URL>` : Target URL.
- `-d, --dir <PATH>` : Base output directory (default: `.`).
- `--release` : Write `release` JSON to `musicbrainz_release.json`.
- `--release-group` : Write `release-group` JSON to `musicbrainz_releasegroup.json`.
- `--all-releases` : Write all-releases JSON to `musicbrainz_all_releases.json`.
- `-f, --force` : Overwrite existing files. Without it, existing paths are skipped; if all target paths exist, the network fetch is skipped entirely.

All-releases JSON is an array containing all `release` responses found in the determined `release-group`.

## How It Works

It reads and validates the URL to determine the type of it. It accepts only `release` or `release-group` URLs. Since the `release` cannot be derived from the `release-group` the CLI has branching logic.

- Parse and validate input URL
- Resolve all target paths, if exist and no `--force`, skip network call -> exit `0`

If `release` URL:

- If `--release` -> fetch `release` and save under a `--release` path
- If `--release-group` -> fetch `release-group` from `release` and save under a `--release-group` path
- If `--all-releases` -> fetch all `release` responses in the determined `release-group` under a `--all-releases` path

If `release-group` URL:

- If `--release` -> ignore it. Emit notice in `stderr`.
- If `--release-group` -> fetch `release-group` and save under a `--release-group` path
- If `--all-releases` -> fetch all `release` responses in the determined `release-group` under a `--all-releases` path

## Examples

```bash
# Save the release response under `./musicbrainz_release.json`
musicbrainz_fetch <RELEASE_URL>
```

```bash
# Save the release-group response under `./musicbrainz_releasegroup.json`
musicbrainz_fetch <RELEASE_GROUP_URL>
```

```bash
# Save the release response under `<PATH>/musicbrainz_release.json`
musicbrainz_fetch <RELEASE_URL> -d <PATH>
```

```bash
# Save the release response under `./musicbrainz_release.json`
musicbrainz_fetch <RELEASE_URL> --release
```

```bash
# Save the release-group response under `./musicbrainz_releasegroup.json`
musicbrainz_fetch <RELEASE_GROUP_URL> --release-group
```

```bash
# Save the release response under `<PATH>/musicbrainz_release.json`
# Save the release-group response derived from the release under `<PATH>/musicbrainz_releasegroup.json`
# Save all releases inside derived release-group in array under `<PATH>/musicbrainz_all_releases.json`
musicbrainz_fetch --dir <PATH> --release --release-group --all-releases --url <RELEASE_URL>
```

```bash
# Derive release-group from the release URL
# Save all releases inside derived release-group in array under `<PATH>/musicbrainz_all_releases.json`
musicbrainz_fetch --dir <PATH> --all-releases --url <RELEASE_URL>
```

```bash
# Cannot be done. Emit notice in `stderr`, exit `1`
musicbrainz_fetch <RELEASE_GROUP_URL> --release
```

```bash
# Save the `release-group` response under `./musicbrainz_releasegroup.json`
# Ignore `--release`, emit notice in `stderr`, exit `0`
musicbrainz_fetch <RELEASE_GROUP_URL> --release --release-group --all-releases
```

