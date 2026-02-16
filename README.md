# Vellum

Vellum is a library manager and MPD client that treats each album as a structured data object. It uses a sidecar metadata architecture to decouple user-managed tags from the underlying audio files, enabling deterministic library management and version-controlled metadata.

## The Album as a Data Object

Vellum represents each album as a data object compiled from several data sources. The final state of an album is resolved by a compiler that merges:

1.  **Manual Metadata**: User-defined tags and identifiers stored in a `metadata.toml` sidecar file.
2.  **Audio Properties**: Technical data (bitrate, sample rate, duration, mtime) harvested directly from audio headers.
3.  **Computed Attributes**: Dynamic data derived during compilation via Python extensions.

### Computable Data
The compilation step generates and maintains metadata that exists independently of the audio file headers or the manual manifest. This allows the system to compute and store data points for an album object such as:
*   Perceptual colorfulness (chroma) and visual complexity (entropy) of cover art.
*   Deterministic library paths and unique IDs based on custom logic.
*   Extended date formatting and historical "date added" priority chains.
*   ReplayGain statistics and audio integrity validation.

## Architecture

*   **Harvester (Rust)**: A high-performance utility used to extract raw tags and technical properties from audio files.
*   **Compiler (Python)**: Resolves the final state of each album object and generates a `metadata.lock.json` artifact.
*   **Server (Rust)**: An Axum-based backend that manages the live library state, monitors the filesystem for changes, and interfaces with `mpd`.
*   **Interface (Svelte 5)**: A reactive web UI.

## CLI Usage

The `vellum` utility provides the following entry points:

| Command | Function |
| :--- | :--- |
| `ui` | Starts the Svelte development server |
| `server` | Starts the Rust backend and MPD monitor |
| `update` | Compiles metadata locks and hot-reloads the server |
| `generate` | Scans for unmanaged audio and creates initial TOML manifests |
| `write` | Synchronizes compiled metadata back into audio file headers |
| `harvest` | Outputs raw file metadata as JSON |
| `report` | Generates listening reports from ListenBrainz export data |

## Development

Vellum provides a Nix flake for a reproducible development environment, including all necessary Python, Rust, and Node.js dependencies.

```bash
nix develop
vellum --help
